use std::path::PathBuf;

use thiserror::Error;

use crate::challenge::{ChallengeCaseError, ChallengeExecutionError, ChallengeParseError};

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
    #[error(transparent)]
    ParseError(#[from] ChallengeParseError),
    #[error(transparent)]
    InputCaseError(#[from] ChallengeCaseError),
    #[error(transparent)]
    ExecutionError(#[from] ChallengeExecutionError),
}
