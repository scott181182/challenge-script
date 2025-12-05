use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

mod command;
mod errors;
mod misc;
mod parsing;

pub use crate::challenge::command::CommandConfig;
use crate::challenge::parsing::ChallengeConfigData;
use crate::template::template_string;

pub use self::errors::{
    ChallengeCaseError, ChallengeExecutionError, ChallengeParseError, CommandParseError,
    StringReferenceError,
};
pub use self::misc::StringReference;

#[derive(Debug, Clone)]
pub struct ChallengeExpectation {
    pub stdout: StringReference,
}

#[derive(Debug, Clone)]
pub struct ChallengeCase {
    pub name: String,
    parent_name: String,
    config: CommandConfig,
    stdin: Option<StringReference>,
    expected: Option<ChallengeExpectation>,
}
impl ChallengeCase {
    pub fn execute<P: AsRef<Path>>(
        self,
        challenge_dir: P,
        command: &ChallengeCommand,
    ) -> Result<(), ChallengeExecutionError> {
        let mut cmd = command.get_command(&self.parent_name, &self.name)?;
        if let Some(args) = self.config.arguments {
            cmd.args(args);
        }
        cmd.current_dir(&challenge_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        if let Some(env_vars) = &self.config.environment {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let mut child = match &self.stdin {
            Some(StringReference::Immediate(s)) => {
                cmd.stdin(Stdio::piped());
                let child = cmd.spawn().map_err(ChallengeExecutionError::SpawnFailed)?;
                let mut child_stdin = child
                    .stdin
                    .as_ref()
                    .ok_or(ChallengeExecutionError::ClosedStdin)?;
                child_stdin
                    .write_all(s.as_bytes())
                    .map_err(ChallengeExecutionError::CouldNotWriteStdin)?;
                child
            }
            Some(StringReference::File { file }) => {
                let filepath = challenge_dir.as_ref().join(file);
                cmd.stdin(File::open(filepath).map_err(StringReferenceError::FileRead)?);
                cmd.spawn().map_err(ChallengeExecutionError::SpawnFailed)?
            }
            None => cmd.spawn().map_err(ChallengeExecutionError::SpawnFailed)?,
        };

        child.wait()?;

        let mut child_out = child.stdout.ok_or(ChallengeExecutionError::ClosedStdout)?;
        let mut output = String::new();
        child_out
            .read_to_string(&mut output)
            .map_err(ChallengeExecutionError::CouldNotReadStdout)?;

        println!("{output}");

        if let Some(expected) = self.expected {
            let actual_output = output.trim();
            let expected_content: String = expected.stdout.into_string(challenge_dir)?;
            if expected_content.trim() != actual_output {
                return Err(ChallengeExecutionError::UnexpectedOutput {
                    expected: expected_content,
                    actual: actual_output.to_owned(),
                });
            } else {
                println!("Matched expected output!");
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ChallengeCommandScript {
    Shell(String),
    Exec(Vec<String>),
}
#[derive(Debug, Clone)]
pub struct ChallengeCommand {
    script: ChallengeCommandScript,
    template: bool,
}
impl ChallengeCommand {
    pub fn get_command(
        &self,
        part_name: &str,
        case_name: &str,
    ) -> Result<Command, CommandParseError> {
        let mut command_array: VecDeque<String> = match &self.script {
            ChallengeCommandScript::Shell(s) => shlex::split(s)
                .ok_or(CommandParseError::MalformedString(s.clone()))?
                .into(),
            ChallengeCommandScript::Exec(l) => l.clone().into(),
        };

        if self.template {
            let context = HashMap::from([("part", part_name), ("case", case_name)]);

            command_array = command_array
                .into_iter()
                .map(|c| template_string(&c, &context))
                .collect();
        }

        let program = command_array
            .pop_front()
            .ok_or(CommandParseError::EmptyCommand)?;

        let mut cmd = Command::new(program);
        cmd.args(command_array);
        Ok(cmd)
    }
}

#[derive(Debug)]
pub struct ChallengeConfigGroup {
    pub name: String,
    parts: Vec<ChallengeConfig>,
}
#[derive(Debug)]
pub struct ChallengeConfigPart {
    pub name: String,
    pub command: ChallengeCommand,
    cases: Vec<ChallengeCase>,
}

#[derive(Debug)]
pub enum ChallengeConfig {
    Group(ChallengeConfigGroup),
    Part(ChallengeConfigPart),
}
impl ChallengeConfig {
    pub fn parse_file<R: std::io::Read>(reader: R) -> Result<Self, ChallengeParseError> {
        let data: ChallengeConfigData = serde_yaml::from_reader(reader)?;
        data.try_into()
    }

    pub fn get_name(&self) -> &str {
        match self {
            ChallengeConfig::Group(c) => &c.name,
            ChallengeConfig::Part(c) => &c.name,
        }
    }

    pub fn resolve_case<I: Iterator<Item = String>>(
        &self,
        mut cases: I,
        config: CommandConfig,
    ) -> Result<(ChallengeCommand, ChallengeCase), ChallengeCaseError> {
        let next_part = cases.next().ok_or(ChallengeCaseError::NotEnoughCases)?;
        match self {
            ChallengeConfig::Group(parent) => {
                let next_config = parent
                    .parts
                    .iter()
                    .find(|c| c.get_name() == next_part)
                    .ok_or_else(|| ChallengeCaseError::CaseNotFound {
                        case: next_part,
                        config_name: parent.name.clone(),
                    })?;
                next_config.resolve_case(cases, config)
            }
            ChallengeConfig::Part(leaf) => {
                let mut case = leaf
                    .cases
                    .iter()
                    .find(|c| c.name == next_part)
                    .ok_or_else(|| ChallengeCaseError::CaseNotFound {
                        case: next_part,
                        config_name: leaf.name.clone(),
                    })
                    .cloned()?;

                case.config = config.merge(&case.config);

                Ok((leaf.command.clone(), case))
            }
        }
    }
}
