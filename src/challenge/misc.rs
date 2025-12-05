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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bad_string_file_reference() {
        let cwd = std::env::current_dir().expect("Couldn't get current working directory");
        let str_ref = StringReference::File {
            file: "fake_file".to_owned(),
        };

        let str_res = str_ref.into_string(cwd);

        assert!(str_res.is_err());
    }
}
