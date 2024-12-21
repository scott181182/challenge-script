use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};

use serde_yaml::Error as YamlError;
use thiserror::Error;

pub mod challenge;
use challenge::{ChallengeCaseError, ChallengeConfig, ChallengeExecutionError};



#[derive(Debug, Error)]
pub enum ChallengeFileError {
    #[error("Could not find directory or challenge file at '{0}'")]
    FileDoesNotExist(PathBuf),
    #[error("Could not find parent directory of '{0}'")]
    CouldNotFindParent(PathBuf),
    #[error("Could not find challenge.yml or challenge.yaml in '{0}'")]
    FileNotFoundInDirectory(PathBuf),

    #[error(transparent)]
    CouldNotOpenFile(std::io::Error),
    #[error(transparent)]
    CouldNotReadDirectory(std::io::Error),
}

fn get_challenge_file<P: AsRef<Path>>(input: P) -> Result<(PathBuf, File), ChallengeFileError> {
    let input_path = input.as_ref();
    if input_path.is_file() {
        let parent = input_path.parent().ok_or(ChallengeFileError::CouldNotFindParent(input_path.to_owned()))?;
        let file = File::open(input_path).map_err(ChallengeFileError::CouldNotOpenFile)?;
        Ok((parent.to_owned(), file))
    } else if input_path.is_dir() {
        let mut dir_reader = read_dir(input_path)
            .map_err(ChallengeFileError::CouldNotReadDirectory)?;
        let challenge_file = dir_reader.find_map(|res| {
            if let Ok(ent) = res {
                if ent.file_name() == "challenge.yml" || ent.file_name() == "challenge.yaml" {
                    return Some(ent.path());
                }
            }
            None
        }).ok_or_else(|| ChallengeFileError::FileNotFoundInDirectory(input_path.to_owned()))?;
        let file = File::open(challenge_file).map_err(ChallengeFileError::CouldNotOpenFile)?;

        Ok((input_path.to_owned(), file))
    } else {
        Err(ChallengeFileError::FileDoesNotExist(input_path.to_owned()))
    }
}



#[derive(Debug, Error)]
pub enum ProgramError {
    #[error(transparent)]
    InputFileError(#[from] ChallengeFileError),
    #[error("Failed to parse YAML file")]
    ParseError(#[from] YamlError),
    #[error(transparent)]
    InputCaseError(#[from] ChallengeCaseError),
    #[error(transparent)]
    ExecutionError(#[from] ChallengeExecutionError),
}



pub fn run_challenge<P: AsRef<Path>>(challenge_path: P, cases: Vec<String>) -> Result<(), ProgramError> {
    let (challenge_dir, challenge_file) = get_challenge_file(challenge_path)?;
    let challenge_config: ChallengeConfig = serde_yaml::from_reader(challenge_file)?;
    let (command, case) = challenge_config.get_case(cases.into_iter())?;
    case.execute(challenge_dir, &command)?;

    Ok(())
}
