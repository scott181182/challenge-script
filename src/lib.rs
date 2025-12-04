use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};

pub mod challenge;
mod errors;

pub use self::errors::{ChallengeFileError, ProgramError};
use crate::challenge::{ChallengeConfig, CommandConfig};

fn get_challenge_file<P: AsRef<Path>>(input: P) -> Result<(PathBuf, File), ChallengeFileError> {
    let input_path = input.as_ref();
    if input_path.is_file() {
        let parent = input_path
            .parent()
            .ok_or(ChallengeFileError::CouldNotFindParent(
                input_path.to_owned(),
            ))?;
        let file = File::open(input_path).map_err(ChallengeFileError::CouldNotOpenFile)?;
        Ok((parent.to_owned(), file))
    } else if input_path.is_dir() {
        let mut dir_reader =
            read_dir(input_path).map_err(ChallengeFileError::CouldNotReadDirectory)?;
        let challenge_file = dir_reader
            .find_map(|res| {
                if let Ok(ent) = res {
                    if ent.file_name() == "challenge.yml" || ent.file_name() == "challenge.yaml" {
                        return Some(ent.path());
                    }
                }
                None
            })
            .ok_or_else(|| ChallengeFileError::FileNotFoundInDirectory(input_path.to_owned()))?;
        let file = File::open(challenge_file).map_err(ChallengeFileError::CouldNotOpenFile)?;

        Ok((input_path.to_owned(), file))
    } else {
        Err(ChallengeFileError::FileDoesNotExist(input_path.to_owned()))
    }
}

pub fn run_challenge<P: AsRef<Path>>(
    challenge_path: P,
    cases: Vec<String>,
) -> Result<(), ProgramError> {
    let (challenge_dir, challenge_file) = get_challenge_file(challenge_path)?;
    let challenge_config: ChallengeConfig = serde_yaml::from_reader(challenge_file)?;
    let (command, case) =
        challenge_config.resolve_case(cases.into_iter(), CommandConfig::default())?;
    case.execute(challenge_dir, &command)?;

    Ok(())
}
