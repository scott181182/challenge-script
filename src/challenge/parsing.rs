use serde::Deserialize;

use super::misc::StringReference;
use crate::challenge::command::CommandConfig;
use crate::challenge::{
    ChallengeCase, ChallengeCommand, ChallengeConfig, ChallengeConfigGroup, ChallengeConfigPart,
    ChallengeExpectation, ChallengeParseError,
};

trait TryResolveChallenge<T> {
    fn try_resolve(
        self,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<T, ChallengeParseError>;
}
trait TryResolveCase<T> {
    fn try_resolve(self, inherit_config: CommandConfig) -> Result<T, ChallengeParseError>;
}
macro_rules! resolve_into {
    ($data:ident, $parsed:ident) => {
        impl TryInto<$parsed> for $data {
            type Error = ChallengeParseError;

            fn try_into(self) -> Result<$parsed, Self::Error> {
                self.try_resolve(None, CommandConfig::default())
            }
        }
    };
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeExpectationData {
    pub stdout: StringReference,
}
impl Into<ChallengeExpectation> for ChallengeExpectationData {
    fn into(self) -> ChallengeExpectation {
        ChallengeExpectation {
            stdout: self.stdout,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChallengeCaseData {
    pub name: String,
    #[serde(flatten)]
    config: CommandConfig,
    stdin: Option<StringReference>,
    expected: Option<ChallengeExpectationData>,
}
impl TryResolveCase<ChallengeCase> for ChallengeCaseData {
    fn try_resolve(
        self,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeCase, ChallengeParseError> {
        Ok(ChallengeCase {
            name: self.name,
            config: inherit_config.merge(&self.config),
            stdin: self.stdin,
            expected: self.expected.map(ChallengeExpectationData::into),
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ChallengeCommandData {
    Immediate(String),
    List(Vec<String>),
}
impl Into<ChallengeCommand> for ChallengeCommandData {
    fn into(self) -> ChallengeCommand {
        match self {
            ChallengeCommandData::Immediate(imm) => ChallengeCommand::Immediate(imm),
            ChallengeCommandData::List(items) => ChallengeCommand::List(items),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChallengeConfigGroupData {
    pub name: String,
    pub command: Option<ChallengeCommandData>,
    #[serde(flatten)]
    config: CommandConfig,

    parts: Vec<ChallengeConfigData>,
}
impl TryResolveChallenge<ChallengeConfigGroup> for ChallengeConfigGroupData {
    fn try_resolve(
        self,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeConfigGroup, ChallengeParseError> {
        Ok(ChallengeConfigGroup {
            name: self.name,
            parts: self
                .parts
                .into_iter()
                .map(|part_data| {
                    part_data.try_resolve(
                        self.command.clone().or(inherit_command.clone()),
                        inherit_config.merge(&self.config),
                    )
                })
                .collect::<Result<_, _>>()?,
        })
    }
}
resolve_into!(ChallengeConfigGroupData, ChallengeConfigGroup);

#[derive(Debug, Deserialize)]
pub struct ChallengeConfigPartData {
    pub name: String,
    pub command: Option<ChallengeCommandData>,
    #[serde(flatten)]
    config: CommandConfig,

    cases: Vec<ChallengeCaseData>,
}
impl TryResolveChallenge<ChallengeConfigPart> for ChallengeConfigPartData {
    fn try_resolve(
        self,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeConfigPart, ChallengeParseError> {
        Ok(ChallengeConfigPart {
            name: self.name.clone(),
            command: self
                .command
                .or(inherit_command)
                .ok_or(ChallengeParseError::NoCommandFound(self.name))?
                .into(),
            cases: self
                .cases
                .into_iter()
                .map(|case_data| case_data.try_resolve(inherit_config.merge(&self.config)))
                .collect::<Result<_, _>>()?,
        })
    }
}
resolve_into!(ChallengeConfigPartData, ChallengeConfigPart);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ChallengeConfigData {
    Group(ChallengeConfigGroupData),
    Part(ChallengeConfigPartData),
}
impl TryResolveChallenge<ChallengeConfig> for ChallengeConfigData {
    fn try_resolve(
        self,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeConfig, ChallengeParseError> {
        match self {
            ChallengeConfigData::Group(group) => Ok(ChallengeConfig::Group(
                group.try_resolve(inherit_command, inherit_config)?,
            )),
            ChallengeConfigData::Part(part) => Ok(ChallengeConfig::Part(
                part.try_resolve(inherit_command, inherit_config)?,
            )),
        }
    }
}
resolve_into!(ChallengeConfigData, ChallengeConfig);
