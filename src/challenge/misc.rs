use std::{fs::read_to_string, path::Path};

use serde::Deserialize;

use crate::challenge::StringReferenceError;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringReference {
    Immediate(String),
    File { file: String },
}
impl StringReference {
    pub fn into_string<P: AsRef<Path>>(
        self,
        challenge_dir: P,
    ) -> Result<String, StringReferenceError> {
        Ok(match self {
            StringReference::Immediate(s) => s,
            StringReference::File { file } => {
                let filepath = challenge_dir.as_ref().join(file);
                read_to_string(filepath)?
            }
        })
    }
}
