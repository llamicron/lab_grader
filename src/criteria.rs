//! A collection of [`Criterion`](crate::criterion::Criterion)
//!
//! This is basically a fancy `Vec<Criterion>`. It implements `FromIterator` so you
//! can `collect()` an iterator of criterions into criteria.
//!
//! ## Example
//! ```rust
//! use lab_grader::*;
//!
//! let criteria = Criteria::from(vec! [
//!     Criterion::new("test 1", 15, ("p", "f"), Box::new(|_: &TestData| true)),
//!     Criterion::new("test 2", 10, ("p", "f"), Box::new(|_: &TestData| false)),
//! ]);
//!
//! assert!(criteria.len() == 2);
//! ```
//! **Or**
//! ```rust
//! // same as above..
//! # use lab_grader::*;
//! #
//! # let loose = vec! [
//! #     Criterion::new("test 1", 15, ("p", "f"), Box::new(|_: &TestData| true)),
//! #     Criterion::new("test 2", 10, ("p", "f"), Box::new(|_: &TestData| false)),
//! # ];
//!
//! let criteria: Criteria = loose.into_iter().collect();
//! assert!(criteria.len() == 2);
//! ```

use std::fmt;
use std::process::exit;
use std::iter::FromIterator;

use crate::{Criterion, TestData};

/// The Criteria struct, just a collection of [`Criterion`](crate::criterion::Criterion)
pub struct Criteria(pub Vec<Criterion>);


impl Criteria {
    // Creates a new empty criteria
    fn new() -> Criteria {
        Criteria(Vec::new())
    }

    /// Add a `Criterion` to the collection
    pub fn add(&mut self, criterion: Criterion) {
        self.0.push(criterion);
    }

    /// Gets `Some(Criterion)` that has the given stub. Returns
    /// `None` if there isn't one at that index
    ///
    /// ## Example
    /// ```rust
    /// # use lab_grader::{Criteria, Criterion, TestData};
    /// #
    /// let mut crit = Criterion::new("test criterion", 1, ("passed", "failed"), Box::new(|_: &TestData| true));
    /// crit.stub = String::from("test-crit-1");
    /// let mut criteria = Criteria::from(vec![crit]);
    ///
    /// assert!(criteria.get("test-crit-1").is_some());
    /// assert!(criteria.get("doesnt-exist").is_none());
    /// ```
    pub fn get(&mut self, stub: &str) -> Option<&mut Criterion> {
        self.0.iter_mut().find(|c| c.stub == stub )
    }

    pub fn attach(&mut self, stub: &str, func: Box<dyn Fn(&TestData) -> bool>) {
        match self.get(stub) {
            Some(crit) => crit.attach(func),
            None => {
                eprintln!("Couldn't find criterion with stub {}", stub);
                exit(1);
            }
        }
    }

    /// Creates a `Criteria` collection from a `Vec<Criterion>`
    ///
    /// ## Example
    /// ```rust
    /// use lab_grader::*;
    ///
    /// let critiera = Criteria::from(vec![
    ///     Criterion::new("name", 1, ("p", "f"), Box::new(|_: &TestData| true))
    /// ]);
    /// ```
    pub fn from(criteria: Vec<Criterion>) -> Self {
        criteria.into_iter().collect()
    }

    /// Returns the amount of `Criterion`s in this collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the total points value of all the criteria
    ///
    /// This is *not* a grade, but the maximum possible grade.
    /// Only a [`Submission`](crate::submission::Submission) holds a grade.
    pub fn total_points(&self) -> usize {
        let mut total: usize = 0;
        for crit in &self.0 {
            total += crit.worth as usize;
        }
        total
    }
}

impl FromIterator<Criterion> for Criteria {
    fn from_iter<I: IntoIterator<Item=Criterion>>(iter: I) -> Self {
        let mut criteria = Criteria::new();

        for i in iter {
            criteria.add(i);
        }

        criteria
    }
}


impl fmt::Display for Criteria {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for crit in &self.0 {
            writeln!(f, "{}", crit).unwrap();
        }
        write!(f, "")
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestData;

    #[test]
    fn test_build_criteria() {
        let loose = vec![
            Criterion::new("test 1", 1, ("p", "f"), Box::new(|_: &TestData| true)),
            Criterion::new("test 2", 1, ("p", "f"), Box::new(|_: &TestData| true)),
        ].into_iter();
        let criteria: Criteria = loose.collect();
        assert!(criteria.0.len() == 2);
    }

    #[test]
    fn test_build_from_vec() {
        let criteria = Criteria::from(vec![
            Criterion::new("test 1", 1, ("p", "f"), Box::new(|_: &TestData| true)),
            Criterion::new("test 2", 1, ("p", "f"), Box::new(|_: &TestData| true)),
        ]);
        assert!(criteria.0.len() == 2);
    }

    #[test]
    fn test_len() {
        let criteria = Criteria::from(vec![
            Criterion::new("test 1", 1, ("p", "f"), Box::new(|_: &TestData| true)),
            Criterion::new("test 2", 1, ("p", "f"), Box::new(|_: &TestData| true)),
        ]);
        assert!(criteria.len() == 2);
        assert!(criteria.0.len() == criteria.len());
    }

    #[test]
    fn test_add_criterion() {
        let mut criteria = Criteria::from(vec![
            Criterion::new("test 1", 1, ("p", "f"), Box::new(|_: &TestData| true)),
            Criterion::new("test 2", 1, ("p", "f"), Box::new(|_: &TestData| true)),
        ]);

        assert!(criteria.len() == 2);

        criteria.add(Criterion::new(
            "test 3", 1, ("p", "f"), Box::new(|_: &TestData| false)
        ));

        assert!(criteria.len() == 3);
    }

    #[test]
    fn test_total_points() {
        let criteria = Criteria::from(vec![
            Criterion::new("test 1", 10, ("p", "f"), Box::new(|_: &TestData| true)),
            Criterion::new("test 2", 25, ("p", "f"), Box::new(|_: &TestData| true)),
        ]);
        assert!(criteria.total_points() == 35);
    }

    #[test]
    fn test_get_criterion() {
        let expected = "test 1";
        let mut crit1 = Criterion::new("test 1", 10, ("p", "f"), Box::new(|_: &TestData| true));
        crit1.stub = String::from("test1");
        let mut crit2 = Criterion::new("test 2", 25, ("p", "f"), Box::new(|_: &TestData| true));
        crit2.stub = String::from("test2");
        let mut criteria = Criteria::from(vec![crit1, crit2]);
        if let Some(found) = criteria.get("test1") {
            assert_eq!(found.name, expected);
        }
    }
}
