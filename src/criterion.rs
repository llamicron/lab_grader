//! Definitions for creating and running criteria
//!
//! A criterion is one specific item in a series of items that form a grade.
//! Each criterion has a name, point value, and a related function.
//! Testing the criterion is running the function, which will return true
//! or false. A final grade can be calculated by adding up all the values
//! of the criteria, if they passed.
//!
//! You **probably shouldn't** create criteria individually through this module,
//! but you can if you want. Instead, you should define your criteria in `YAML` then
//! build that into a [`Batch`](crate::batch::Batch).

use std::fmt;
use std::fmt::Write;

use ansi_term::Color::{Green, Red, White};

use crate::TestData;


// TODO: Move this to submission.rs
/// A macro to easily create a `TestData` struct, which is
/// really just an alias to `HashMap<String, String>`
///
/// ## Example
/// ```rust
/// # #[macro_use] extern crate lab_grader;
/// use lab_grader::TestData;
///
/// // The long way
/// let mut map = TestData::new();
/// map.insert(String::from("key"), String::from("value"));
///
/// // the macro way
/// let data = data! { "key" => "value" };
/// assert_eq!(map, data);
/// ```
#[macro_export]
macro_rules! data(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert(String::from($key), String::from($value));
            )+
            m
        }
     };
);

/// A single Criterion
pub struct Criterion {
    /// An ID stub used to identify this criterion
    pub stub: String,
    /// A short (< 30 characters), descriptive name
    pub name: String,
    /// Point value of this criterion. If it passes, this value
    /// will be added to the [`Submission`](crate::submission::Submission) grade.
    ///
    /// Can be negative if you wish to subtract points. Be sure to get your logic right.
    /// This value is added to the submission grade *if the test returns true*.
    pub worth: i16,
    /// Pass or fail messages, respectively
    ///
    /// When printing a criterion, the appropriate message
    /// will be printed. Not much use other than that.
    pub messages: (String, String),
    /// An optional description
    pub desc: String,
    /// The criterion's test
    ///
    /// Determines if the criterion passes or fails. This signature is
    /// required.
    pub test: Box<dyn Fn(&TestData) -> bool>,
    /// If the test passed, failed, or hasn't been run.
    ///
    /// `None` if it hasn't been run, Some(`true`) or Some(`false`) otherwise.
    /// If this value is `Some`, the test has been run.
    pub status: Option<bool>,
    /// Renders the criterion unable to be printed
    pub hide: bool,
}

impl Criterion {
    /// Creates a new Criterion with the given parameters.
    ///
    /// The `messages` parameter should be a tuple of
    /// `&str` containing a success then failure message, respectively.
    /// These messages will be printed when printing the criterion.
    ///
    /// The `test` parameter is a [`Box`][box] around a closure accepting
    /// a reference to [TestData][testdata] returning a bool. This can get a bit confusing.
    /// The `test` closure should return true if the criterion passes, otherwise false.
    /// The `&TestData` parameter allows data from outside the closure to be passed in. `TestData` is
    /// just an alias to `HashMap<String, String>`, so all keys and values must be `String`s.
    ///
    /// [testdata]: crate::submission::TestData
    /// [box]: std::boxed::Box
    ///
    /// ## Example
    /// **A basic criterion**
    /// ```rust
    /// use lab_grader::{Criterion, TestData};
    ///
    /// let mut c = Criterion::new(
    ///     "A test criterion",
    ///     10,
    ///     ("Success!", "Failure!"),
    ///     Box::new(|_: &TestData| {
    ///         // Code to test criterion goes here,
    ///         // and should return false or...
    ///         true
    ///     })
    /// );
    /// assert!(c.test());
    /// ```
    ///
    /// **A criterion with data**
    /// ```rust
    /// # #[macro_use] extern crate lab_grader;
    /// # use lab_grader::{Criterion, TestData};
    ///
    /// let mut c = Criterion::new(
    ///     "A test criterion with data!",
    ///     10,
    ///     ("Success!", "Failure!"),
    ///     Box::new(|data: &TestData| {
    ///         return data["my_key"] == "my_value";
    ///     })
    /// );
    ///
    /// // The above criterion takes a `&TestData` into it's closure,
    /// // so we must establish the data to send into the closure
    /// let my_data = data! {
    ///     "my_key" => "my_value"
    /// };
    ///
    /// assert!(c.test_with_data(&my_data));
    /// ```
    pub fn new<S: AsRef<str>>(
        name: S,
        worth: i16,
        messages: (S, S),
        test: Box<dyn Fn(&TestData) -> bool>,
    ) -> Self {
        Criterion {
            stub: String::from("none"),
            name: String::from(name.as_ref()),
            worth,
            messages: (String::from(messages.0.as_ref()), String::from(messages.1.as_ref())),
            desc: String::new(),
            test,
            status: None,
            hide: false,
        }
    }

    /// Sets the description
    pub fn set_desc<S: AsRef<str>>(&mut self, desc: S) {
        self.desc = desc.as_ref().to_string()
    }

    /// Returns the success message, ie. the first message in the [`messages`][msg] tuple
    ///
    /// [msg]: Criterion::new
    pub fn success_message(&self) -> &String {
        &self.messages.0
    }

    /// Returns the failure message, ie. the second message in the [`messages`][msg] tuple
    ///
    /// [msg]: Criterion::new
    pub fn failure_message(&self) -> &String {
        &self.messages.1
    }


    /// Sets the `hide` field on a criterion
    ///
    /// If hide is true, printing the criterion with the default
    /// formatter will print nothing. Good if you want a secret criterion
    /// that the students don't know about
    pub fn hide(&mut self, state: bool) {
        self.hide = state;
    }

    /// Sets the test method of a criterion
    pub fn attach(&mut self, test: Box<dyn Fn(&TestData) -> bool>) {
        self.test = test
    }

    /// Runs the criterion's test function with the data provided.
    ///
    /// This is almost equivilent to calling `(criterion.test)(data)`, but this
    /// method also sets the status of the criterion to the result of the test.
    /// You should avoid calling the test directly, and call this or the
    /// [`test`](Criterion::test) method instead.
    ///
    /// The criterion must be mutable to call this method, as the status is changed
    /// to the result of the test.
    ///
    /// ## Example
    /// ```rust
    /// # #[macro_use] extern crate lab_grader;
    /// # use lab_grader::{Criterion, TestData};
    ///
    /// let mut c = Criterion::new(
    ///     "A test criterion with data!",
    ///     10,
    ///     ("Success!", "Failure!"),
    ///     Box::new(|data: &TestData| {
    ///         return data["my_key"] == "my_value";
    ///     })
    /// );
    ///
    /// let my_data = data! {
    ///     "my_key" => "my_value"
    /// };
    ///
    /// c.test_with_data(&my_data);
    /// // It's either Some(true) or Some(false) since we've tested
    /// assert!(c.status.is_some());
    /// ```
    pub fn test_with_data(&mut self, data: &TestData) -> bool {
        self.status = Some((self.test)(data));
        self.status.unwrap()
    }

    /// Runs the criterions test and assigns the result to `criterion.status`.
    ///
    /// This is equivilent to running [`test_with_data`](crate::criterion::Criterion::test_with_data) with
    /// an empty `TestData`.
    ///
    /// Criterion must be mutable.
    ///
    /// ## Example
    /// ```rust
    /// # use lab_grader::{Criterion, TestData};
    ///
    /// let mut c = Criterion::new(
    ///     "A test criterion with data!",
    ///     10,
    ///     ("Success!", "Failure!"),
    ///     Box::new(|_: &TestData| {
    ///         true
    ///     })
    /// );
    ///
    /// assert!(c.test());
    /// assert!(c.status.is_some());
    /// ```
    pub fn test(&mut self) -> bool {
        self.test_with_data(&TestData::new())
    }

}

/// Displays the results of the criterion.
/// You should test the criterion before printing it.
impl fmt::Display for Criterion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.hide {
            return write!(f, "");
        }
        let mut buffer = String::new();
        if let Some(status) = self.status {
            if status {
                // Success
                writeln!(&mut buffer, "{}", Green.bold().paint(&self.name)).unwrap();
                writeln!(&mut buffer, "{}", White.paint(&self.desc)).unwrap();
                writeln!(&mut buffer, "Worth: {} pts", self.worth).unwrap();
                writeln!(&mut buffer, "Status: {}", Green.paint(self.success_message())).unwrap();
            } else {
                // Failure
                writeln!(&mut buffer, "{}", Red.bold().paint(&self.name)).unwrap();
                writeln!(&mut buffer, "{}", White.paint(&self.desc)).unwrap();
                writeln!(&mut buffer, "Worth: {} pts", self.worth).unwrap();
                writeln!(&mut buffer, "Status: {}", Red.paint(self.failure_message())).unwrap();
            }
        } else {
            // Neutral
            writeln!(&mut buffer, "{}", White.bold().paint(&self.name)).unwrap();
            writeln!(&mut buffer, "{}", White.paint(&self.desc)).unwrap();
            writeln!(&mut buffer, "Worth: {} pts", self.worth).unwrap();
            writeln!(&mut buffer, "Status: not tested").unwrap();
        }
        write!(f, "{}", buffer)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_criterion() {
        let mut c = Criterion::new(
            "A test criterion",
            10,
            ("passed!", "failed!"),
            Box::from(|_: &TestData| -> bool { true }),
        );
        assert_eq!(c.name, "A test criterion");
        assert_eq!(c.worth, 10);
        assert!(c.status.is_none());
        assert!(c.test());
        assert!(c.status.is_some());
    }

    #[test]
    fn test_a_criterion_with_data_passes() {
        let mut c = Criterion::new(
            "A test criterion",
            10,
            ("succes!", "failure!"),
            Box::from(|data: &TestData| -> bool {
                return data["my_var"] == "value";
            }),
        );

        let data = data! {
            "my_var" => "value"
        };

        assert!(c.test_with_data(&data));
    }

    #[test]
    fn test_success_and_failure_messages() {
        let c = Criterion::new(
            "A test criterion",
            10,
            ("passed!", "failed!"),
            Box::from(|_: &TestData| -> bool { true }),
        );
        assert_eq!(c.success_message(), "passed!");
        assert_eq!(c.failure_message(), "failed!");
    }

    #[test]
    fn test_data_macro() {
        // The long way
        let mut map = TestData::new();
        map.insert(String::from("key"), String::from("value"));

        // the macro way
        let data = data! { "key" => "value" };
        assert_eq!(map, data);
    }

    #[test]
    fn test_set_description() {
        let mut c = Criterion::new("test", 1, ("p", "f"), Box::new(|_: &TestData| false));
        assert!(c.desc.len() == 0);
        c.set_desc("short desc");
        assert_eq!(c.desc, "short desc");
    }

    #[test]
    fn test_set_test() {
        let mut c = Criterion::new("test", 1, ("p", "f"), Box::new(|_: &TestData| false));
        assert!(!c.test());

        let new_func = Box::new(|_: &TestData| true);
        c.attach(new_func);
        assert!(c.test());
    }

    #[test]
    fn test_hide_criterion() {
        let mut crit = Criterion::new("test", 1, ("p", "f"), Box::new(|_: &TestData| true));

        assert!(format!("{}", crit).len() > 1);
        crit.hide(true);
        assert_eq!(format!("{}", crit).len(), 0);
    }
}
