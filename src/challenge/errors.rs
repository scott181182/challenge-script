use serde_yaml::Error as YamlError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StringReferenceError {
    #[error(transparent)]
    FileRead(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ChallengeParseError {
    #[error("Failed to parse YAML file: {0}")]
    Yaml(#[from] YamlError),

    #[error("Could not find command for challenge part '{0}'")]
    NoCommandFound(String),
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    #[error("Malformed command string")]
    MalformedString(String),
    #[error("Empty command")]
    EmptyCommand,
}

#[derive(Debug, Error)]
pub enum ChallengeExecutionError {
    #[error(transparent)]
    BadStringReference(#[from] StringReferenceError),
    #[error(transparent)]
    BadCommand(#[from] CommandParseError),

    #[error("Couldn't open stdin of child process")]
    ClosedStdin,
    #[error("Couldn't open stdout of child process")]
    ClosedStdout,

    #[error(transparent)]
    CouldNotWriteStdin(std::io::Error),
    #[error(transparent)]
    CouldNotReadStdout(std::io::Error),
    #[error(transparent)]
    SpawnFailed(std::io::Error),
    #[error(transparent)]
    ExecutionFailed(#[from] std::io::Error),

    #[error("Wrong output. Expected '{expected}' but found '{actual}'")]
    UnexpectedOutput { expected: String, actual: String },
}

#[derive(Debug, Error)]
pub enum ChallengeCaseError {
    #[error("Expected another case, but found none")]
    NotEnoughCases,
    #[error("Could not find case '{case}' in config '{config_name}'")]
    CaseNotFound { case: String, config_name: String },
}
