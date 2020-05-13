//! A bundle of data that criteria are graded against, and is submitted for review

// std uses
use std::collections::HashMap;

// external uses
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// internal uses
use crate::results_file::AsCsv;
use crate::criteria::Criteria;
use crate::server;



/// A type alias to `HashMap<String, String>`
///
/// This is the data type that all criteria accept,
/// and how data is stored in a submission
pub type TestData = HashMap<String, String>;


/// A macro to easily create a [`TestData`](crate::submission::TestData)
/// struct, which is really just an alias to `HashMap<String, String>`.
///
/// ## Example
/// ```rust
/// # extern crate lab_grader;
/// use lab_grader::{TestData, data};
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


/// A submission is a bundle of data that represents
/// one student's submission. They will do some sort of work
/// for a lab, then run a rust script that builds some criteria,
/// runs those criteria with some data from the student, and submits
/// a Submission to a central webserver where the instructor can
/// collect the graded submissions.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Submission {
    /// A local timestamp when the submission was created
    pub time: DateTime<Local>,
    /// Numerical grade for the submission.
    /// Each criterion will add to this grade if it passes.
    pub grade: i16,
    /// Extra data attached to the submission.
    /// Leave it empty if you don't need it
    pub data: TestData,
    /// The criteria (name) that this submission passed
    pub passed: Vec<String>,
    /// The citeria (name) that this submission failed
    pub failed: Vec<String>
}

impl Submission {
    /// Creates a new submission.
    ///
    /// ## Example
    /// ```rust
    /// use lab_grader::Submission;
    ///
    /// // You probably want it to be mutable so
    /// // you can attach data and change the grade
    /// let mut sub = Submission::new();
    ///
    /// assert_eq!(sub.grade, 0);
    /// assert_eq!(sub.data.len(), 0);
    /// ```
    pub fn new() -> Submission {
        Submission {
            time: Local::now(),
            grade: 0,
            data: TestData::new(),
            passed: Vec::new(),
            failed: Vec::new()
        }
    }

    /// Attaches data to a submission
    ///
    /// The data must be a [`TestData`](crate::submission::TestData).
    /// You may want to use the [`data!`](../macro.data.html) macro to make it
    /// easier to establish your data.
    ///
    /// You may be interested in [`Submission::from_data`](crate::submission::Submission::from_data).
    ///
    /// ## Example
    /// ```rust
    /// # use lab_grader::data;
    /// # use lab_grader::Submission;
    /// #
    /// let data = data! {
    ///     "key" => "value",
    ///     "key2" => "value2"
    /// };
    ///
    /// let mut sub = Submission::new();
    /// sub.use_data(data);
    ///
    /// assert_eq!(sub.data["key"], "value");
    /// assert_eq!(sub.data["key2"], "value2");
    /// ```
    pub fn use_data(&mut self, data: TestData) {
        self.data = data;
    }

    /// Creates a new submission and attaches data to it in one step
    ///
    /// ## Example
    /// ```rust
    /// # use lab_grader::{Submission, data};
    ///
    /// let sub = Submission::from_data(data! {
    ///     "name" => "luke i guess",
    ///     "id" => "1234"
    /// });
    ///
    /// assert_eq!(sub.data["id"], "1234");
    /// ```
    pub fn from_data(data: TestData) -> Self {
        let mut sub = Submission::new();
        sub.use_data(data);
        sub
    }

    /// Marks a criterion as passed. Provide the name of the criterion.
    ///
    /// This struct does not include an actual [`Criterion`](crate::criterion::Criterion)
    /// struct in it's `passed` and `failed` fields, because it's impossible to
    /// serialize a `Criterion`. `Submission`s must be serializable. Instead, only the
    /// name and message of the criterion are stored on the submission
    ///
    /// ## Example
    /// ```rust
    /// # use lab_grader::Submission;
    /// let mut sub = Submission::new();
    /// sub.pass("Some criterion name");
    ///
    /// assert!(sub.passed.contains(&"Some criterion name".to_string()));
    /// ```
    pub fn pass<C: AsRef<str>>(&mut self, criterion: C) {
        self.passed.push(criterion.as_ref().to_string());
    }

    /// Same as [`pass`](crate::submission::Submission::pass), but adds to the `failed` vector
    pub fn fail<C: AsRef<str>>(&mut self, criterion: C) {
        self.failed.push(criterion.as_ref().to_string());
    }

    /// Tests a submission against a list of criterion
    ///
    /// The submission's grade will change for every passed criterion,
    /// and every criterion will add it's name and message to the submissions
    /// `passed` or `failed` vectors.
    ///
    /// ## Example
    /// ```rust
    /// # use lab_grader::*;
    /// let mut sub = Submission::from_data(data! {
    ///     "key" => "value"
    /// });
    ///
    /// // Just one criterion here to save space
    /// let mut crits = Criteria::from(vec![
    ///     Criterion::new("test criterion")
    ///         .worth(10)
    ///         .test(Box::new(|data: &TestData| -> bool {
    ///             data["key"] == "value"
    ///         }))
    ///         .build()
    /// ]);
    /// sub.grade_against(&mut crits);
    /// assert_eq!(sub.grade, 10);
    /// assert_eq!(sub.passed.len(), 1);
    /// assert_eq!(sub.failed.len(), 0);
    /// ```
    pub fn grade_against(&mut self, criteria: &mut Criteria) {
        for crit in &mut criteria.sorted().into_iter() {
            crit.test_with_data(&self.data);

            if crit.status.unwrap() {
                self.grade += crit.worth;
                self.pass(format!("{}: {}", crit.name, crit.success_message()));
            } else {
                self.fail(format!("{}: {}", crit.name, crit.failure_message()));
            }
        }
    }


    /// Spins up a webserver to accept submission.
    ///
    /// Accepted submissions will be written to a [`ResultsFile`](crate::results_file::ResultsFile).
    /// The web server will run on the provided port.
    ///
    /// The results file will be placed in the directory you execute the code in,
    /// and be called `submissions.csv`.
    ///
    /// The best way to submit a submission to the server that this function starts is
    /// to call [`post_json`](crate::helpers::web::post_json) from the web helpers module and
    /// pass it the url that this server is accessible on, and a submission. It will convert
    /// it to json for you.
    ///
    /// Support for custom results file locations is coming...
    /// ```no_run
    /// use lab_grader::Submission;
    /// Submission::server(8080);
    /// ```
    pub fn server(port: u16) {
        server::run(port);
    }
}

impl AsCsv for TestData {
    /// Returns the test data, serialized to a csv string. It will be
    /// sorted alphabetically by key.
    fn as_csv(&self) -> String {
        let values: Vec<&String> = self.values().collect();
        let mut owned_values: Vec<String> = values.iter().map(|&k| k.to_owned() ).collect();
        owned_values.sort_by(|a,b| a.cmp(&b) );
        return owned_values.join(",");
    }

    /// Returns the filename that the [`ResultsFile`](crate::results_file::ResultsFile)
    /// uses as its output
    ///
    /// This probably shouldn't get used for test data, as it will be written as part
    /// of a submission, not on it's own.
    fn filename(&self) -> String {
        String::from("submission_data.csv")
    }

    /// Returns a header to write to a csv file. This should match the fields in `as_csv` above.
    fn header(&self) -> String {
        let keys: Vec<&String> = self.keys().collect();
        let mut owned_keys: Vec<String> = keys.iter().map(|&k| k.to_owned() ).collect();
        owned_keys.sort_by(|a,b| a.cmp(&b) );
        return format!("{}", owned_keys.join(","));
    }
}

impl AsCsv for Submission {
    /// Returns the submission's values in csv format. The `TestData` atttached will be
    /// sorted alphabetically by key.
    fn as_csv(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.time.to_rfc3339(),
            self.grade,
            self.passed.join(";"),
            self.failed.join(";"),
            self.data.as_csv()
        )
    }

    /// Returns the filename to use when writing submissions to disk
    fn filename(&self) -> String {
        String::from("submissions.csv")
    }

    /// Returns a header of all the fields, matching the data in `as_csv`
    fn header(&self) -> String {
        format!("time,grade,passed,failed,{}", self.data.header())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;
    use crate::Criterion;


    #[test]
    fn test_new_submission() {
        let sub = Submission::new();
        assert!(sub.data.len() == 0);
    }

    #[test]
    fn test_submission_use_data() {
        let data = data! {
            "key" => "value"
        };
        let mut sub = Submission::new();
        sub.use_data(data);
        assert!(sub.data.len() == 1);
        assert_eq!(sub.data["key"], "value");

        let sub2 = Submission::from_data(data! {
            "key" => "value"
        });
        assert_eq!(sub2.data["key"], "value");
    }

    #[test]
    fn test_submission_as_csv() {
        let sub = Submission::from_data(data! { "a" => "v", "b" => "v2" });

        // TestData keys are sorted alphabetically when converting to csv
        assert!((&sub).as_csv().contains("v,v2"));

        // Submission with no data, passes, or failures
        let sub2 = Submission::new();
        let expected = "0,,,";
        assert!((&sub2).as_csv().contains(expected));
    }

    #[test]
    fn test_serialize_deserialize_json() {
        let mut sub = Submission::from_data(data! { "k2" => "v2", "k" => "v" });
        sub.pass("something");
        sub.fail("something");

        // Assert the
        assert!(serde_json::to_string(&sub).unwrap().contains(r#""k2":"v2""#));

        let data = r#"{"time":"2020-05-01T22:23:21.180875-05:00","grade":0,"passed":["something"],"failed":["something"],"data":{"k2":"v2","k":"v"}}"#;
        let built_sub: Submission = serde_json::from_str(data).unwrap();
        assert_eq!(built_sub.grade, sub.grade);
    }

    #[test]
    fn test_grade_against_criteria() {
        let mut sub = Submission::from_data(data! {
            "key" => "value"
        });

        // Just one criterion here to save space
        let mut crits = Criteria::from(vec![
            Criterion::new("test criterion")
                .worth(10)
                .test(Box::new(|data: &TestData| -> bool {
                    data["key"] == "value"
                }))
                .build()
        ]);

        sub.grade_against(&mut crits);
        assert_eq!(sub.grade, 10);
        assert_eq!(sub.passed.len(), 1);
        assert_eq!(sub.failed.len(), 0);
    }

    #[test]
    fn test_test_data_as_csv() {
        let d = data! {
            "b2" => "value2",
            "a1" => "value1"
        };

        let expected_header = "a1,b2";
        let expected_values = "value1,value2";
        let expected_filename = "submission_data.csv";

        assert_eq!(d.header(), expected_header);
        assert_eq!(d.as_csv(), expected_values);
        assert_eq!(d.filename(), expected_filename);
    }
}
