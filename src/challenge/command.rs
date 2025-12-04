use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CommandConfig {
    pub arguments: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
}

impl CommandConfig {
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            arguments: match (&self.arguments, &other.arguments) {
                (Some(sa), Some(oa)) => Some(sa.clone().into_iter().chain(oa.clone()).collect()),
                (sa, None) => sa.clone(),
                (None, oa) => oa.clone(),
            },
            environment: match (&self.environment, &other.environment) {
                (Some(se), Some(oe)) => Some(se.clone().into_iter().chain(oe.clone()).collect()),
                (se, None) => se.clone(),
                (None, oe) => oe.clone(),
            },
        }
    }
}
