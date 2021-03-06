//! YAML representations of some key structs
//!
//! Much bourbon went into the creation of this module.
//!
//! I think a better way to do what this module does is to implement `FromStr`
//! or some similar trait so that serde can (de)serialize. However, this places
//! some restrictions on the format of the YAML. So instead there are "proxy"
//! structs that mirror the structs I want to serialize. These proxy structs have
//! a slightly different structure, which allows the YAML format I want. They are meant
//! to be consumed and transformed into the target structs.
//! 
//! It is very unlikely that you actually need to use these.


// std uses
use std::collections::HashMap;

// external uses
use serde::Deserialize;

// internal uses
use crate::rubric::Criterion;


/// A yaml representation of a [`Rubric`](crate::rubric::Rubric).
///
/// This struct is just used for deserializing YAML. [`Rubric::from_str`](crate::rubric::Rubric::from_str)
/// uses one of these puppies for deserializing then consumes it to build a Rubric.
#[derive(Deserialize)]
pub struct RubricYaml {
    pub name: String,
    pub desc: Option<String>,
    pub criteria: HashMap<String, CriterionYaml>,
    pub total: Option<isize>,
    pub deadline: Option<String>,
    pub final_deadline: Option<String>,
    pub allow_late: Option<bool>,
    pub late_penalty: Option<isize>,
    pub late_penalty_per_day: Option<isize>,
}

/// A yaml representation of [`Criterion`](crate::criterion::Criterion)
///
/// This can be deserialized from valid yaml, then converted into a
/// Criterion with [`into_criterion`](crate::yaml::CriterionYaml::into_criterion).
#[derive(Deserialize)]
pub struct CriterionYaml {
    func: Option<String>,
    index: Option<i64>,
    desc: Option<String>,
    worth: isize,
    messages: Option<(String, String)>,
    hide: Option<bool>,
}

impl CriterionYaml {
    // Normally I would implement FromStr but I can't because I can't attach the `name`,
    // just because of the yaml format. Kinda fucky, I know.
    pub fn into_criterion(self, name: String) -> Criterion {
        // The two required fields
        let mut builder = Criterion::new(&name).worth(self.worth);

        if let Some(msg) = self.messages {
            builder = builder.messages(&msg.0, &msg.1)
        }
        if let Some(func) = self.func {
            builder = builder.func(&func)
        }
        if let Some(h) = self.hide {
            builder = builder.hide(h)
        }
        if let Some(desc) = self.desc {
            builder = builder.desc(&desc)
        }
        if let Some(index) = self.index {
            builder = builder.index(index);
        }

        builder.build()
    }
}
