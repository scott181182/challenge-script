use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;
use thiserror::Error;



#[derive(Debug, Error)]
pub enum StringReferenceError {
    #[error(transparent)]
    FileRead(#[from] std::io::Error)
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
    UnexpectedOutput{expected: String, actual: String},
}



#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringReference {
    Immediate(String),
    File{file: String}
}
impl StringReference {
    fn into_string<P: AsRef<Path>>(self, challenge_dir: P) -> Result<String, StringReferenceError> {
        Ok(match self {
            StringReference::Immediate(s) => s,
            StringReference::File { file } => {
                let filepath = challenge_dir.as_ref().join(file);
                read_to_string(filepath)?
            },
        })
    }
}



#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeExpectation {
    pub stdout: StringReference
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeCase {
    pub name: String,
    stdin: Option<StringReference>,
    arguments: Option<Vec<String>>,
    expected: Option<ChallengeExpectation>
}
impl ChallengeCase {
    pub fn execute<P: AsRef<Path>>(self, challenge_dir: P, command: &ChallengeCommand) -> Result<(), ChallengeExecutionError> {
        let mut cmd = command.get_command()?;
        if let Some(args) = self.arguments {
            cmd.args(args);
        }
        cmd.current_dir(&challenge_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        let mut child = match self.stdin {
            Some(StringReference::Immediate(s)) => {
                cmd.stdin(Stdio::piped());
                let child = cmd.spawn().map_err(ChallengeExecutionError::SpawnFailed)?;
                let mut child_stdin = child.stdin.as_ref().ok_or(ChallengeExecutionError::ClosedStdin)?;
                child_stdin.write_all(s.as_bytes()).map_err(ChallengeExecutionError::CouldNotWriteStdin)?;
                child
            },
            Some(StringReference::File { file }) => {
                let filepath = challenge_dir.as_ref().join(file);
                cmd.stdin(File::open(filepath).map_err(StringReferenceError::FileRead)?);
                cmd.spawn().map_err(ChallengeExecutionError::SpawnFailed)?
            },
            None => {
                cmd.spawn().map_err(ChallengeExecutionError::SpawnFailed)?
            }
        };

        child.wait()?;

        let mut child_out = child.stdout.ok_or(ChallengeExecutionError::ClosedStdout)?;
        let mut output = String::new();
        child_out.read_to_string(&mut output).map_err(ChallengeExecutionError::CouldNotReadStdout)?;

        println!("{}", output);

        if let Some(expected) = self.expected {
            let actual_output = output.trim();
            let expected_content: String = expected.stdout.into_string(challenge_dir)?;
            if expected_content.trim() != actual_output {
                return Err(ChallengeExecutionError::UnexpectedOutput {
                    expected: expected_content,
                    actual: actual_output.to_owned()
                });
            }
        }

        Ok(())
    }
}



#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ChallengeCommand {
    Immediate(String),
    List(Vec<String>),
}
impl ChallengeCommand {
    pub fn get_command(&self) -> Result<Command, CommandParseError> {
        let command_array = match self {
            ChallengeCommand::Immediate(s) => shlex::split(s).ok_or(CommandParseError::MalformedString(s.clone()))?,
            ChallengeCommand::List(l) => l.clone()
        };
        let mut command_iter = command_array.into_iter();
        let program = command_iter.next().ok_or(CommandParseError::EmptyCommand)?;

        let mut cmd = Command::new(program);
        cmd.args(command_iter);
        Ok(cmd)
    }
}



#[derive(Debug, Deserialize)]
pub struct ChallengeConfigParent {
    pub name: String,
    parts: Vec<ChallengeConfig>
}
#[derive(Debug, Deserialize)]
pub struct ChallengeConfigLeaf {
    pub name: String,
    pub command: ChallengeCommand,
    cases: Vec<ChallengeCase>
}



#[derive(Debug, Error)]
pub enum ChallengeCaseError {
    #[error("Expected another case, but found none")]
    NotEnoughCases,
    #[error("Could not find case '{case}' in config '{config_name}'")]
    CaseNotFound{case: String, config_name: String}
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ChallengeConfig {
    Parent(ChallengeConfigParent),
    Leaf(ChallengeConfigLeaf)
}
impl ChallengeConfig {
    pub fn get_name(&self) -> &str {
        match self {
            ChallengeConfig::Parent(c) => &c.name,
            ChallengeConfig::Leaf(c) => &c.name,
        }
    }

    pub fn get_case<
        I: Iterator<Item = String>
    >(&self, mut cases: I) -> Result<(ChallengeCommand, ChallengeCase), ChallengeCaseError> {
        let next_part = cases.next().ok_or(ChallengeCaseError::NotEnoughCases)?;
        match self {
            ChallengeConfig::Parent(parent) => {
                let next_config = parent.parts.iter().find(|c| c.get_name() == &next_part)
                    .ok_or_else(|| ChallengeCaseError::CaseNotFound { case: next_part, config_name: parent.name.clone() })?;
                next_config.get_case(cases)
            },
            ChallengeConfig::Leaf(leaf) => {
                let case = leaf.cases.iter().find(|c| &c.name == &next_part)
                    .ok_or_else(|| ChallengeCaseError::CaseNotFound { case: next_part, config_name: leaf.name.clone() })
                    .map(ChallengeCase::clone)?;

                Ok((leaf.command.clone(), case))
            },
        }
    }
}

