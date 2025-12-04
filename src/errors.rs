use std::path::PathBuf;

use serde_yaml::Error as YamlError;
use thiserror::Error;

use crate::challenge::{ChallengeCaseError, ChallengeExecutionError};

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

#[derive(Debug, Error)]
pub enum ProgramError {
    #[error(transparent)]
    InputFileError(#[from] ChallengeFileError),
    #[error("Failed to parse YAML file: {0}")]
    ParseError(#[from] YamlError),
    #[error(transparent)]
    InputCaseError(#[from] ChallengeCaseError),
    #[error(transparent)]
    ExecutionError(#[from] ChallengeExecutionError),
}
