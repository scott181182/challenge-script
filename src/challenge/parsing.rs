use std::collections::HashMap;

use serde::Deserialize;

use super::misc::StringReference;
use crate::challenge::command::CommandConfig;
use crate::challenge::{
    ChallengeCase, ChallengeCommand, ChallengeCommandScript, ChallengeConfig, ChallengeConfigGroup,
    ChallengeConfigPart, ChallengeExpectation, ChallengeParseError,
};

trait TryResolveChallenge<T>
where
    Self: Sized,
{
    fn try_resolve(
        self,
        name: String,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<T, ChallengeParseError>;

    fn try_resolve_default(self, name: String) -> Result<T, ChallengeParseError> {
        self.try_resolve(name, None, CommandConfig::default())
    }
}
trait TryResolveCase<T> {
    fn try_resolve(
        self,
        case_name: String,
        parent_name: String,
        inherit_config: CommandConfig,
    ) -> Result<T, ChallengeParseError>;
}

#[derive(Debug, Deserialize, Clone)]
struct ChallengeExpectationData {
    stdout: StringReference,
}
impl Into<ChallengeExpectation> for ChallengeExpectationData {
    fn into(self) -> ChallengeExpectation {
        ChallengeExpectation {
            stdout: self.stdout,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct ChallengeCaseData {
    #[serde(flatten)]
    config: CommandConfig,
    stdin: Option<StringReference>,
    expected: Option<ChallengeExpectationData>,
}
impl TryResolveCase<ChallengeCase> for ChallengeCaseData {
    fn try_resolve(
        self,
        case_name: String,
        parent_name: String,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeCase, ChallengeParseError> {
        Ok(ChallengeCase {
            name: case_name,
            parent_name,
            config: inherit_config.merge(&self.config),
            stdin: self.stdin,
            expected: self.expected.map(ChallengeExpectationData::into),
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum ChallengeCommandScriptData {
    Shell(String),
    Exec(Vec<String>),
}
impl Into<ChallengeCommandScript> for ChallengeCommandScriptData {
    fn into(self) -> ChallengeCommandScript {
        match self {
            ChallengeCommandScriptData::Shell(s) => ChallengeCommandScript::Shell(s),
            ChallengeCommandScriptData::Exec(s) => ChallengeCommandScript::Exec(s),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct ChallengeCommandObjectData {
    script: ChallengeCommandScriptData,
    template: Option<bool>,
}
impl Into<ChallengeCommand> for ChallengeCommandObjectData {
    fn into(self) -> ChallengeCommand {
        ChallengeCommand {
            script: self.script.into(),
            template: self.template.unwrap_or(true),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum ChallengeCommandData {
    Shell(String),
    Exec(Vec<String>),
    Object(ChallengeCommandObjectData),
}

impl Into<ChallengeCommand> for ChallengeCommandData {
    fn into(self) -> ChallengeCommand {
        match self {
            ChallengeCommandData::Shell(s) => ChallengeCommand {
                script: ChallengeCommandScript::Shell(s),
                template: true,
            },
            ChallengeCommandData::Exec(s) => ChallengeCommand {
                script: ChallengeCommandScript::Exec(s),
                template: true,
            },
            ChallengeCommandData::Object(obj) => obj.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChallengeConfigGroupData {
    command: Option<ChallengeCommandData>,
    #[serde(flatten)]
    config: CommandConfig,

    parts: HashMap<String, ChallengeConfigNode>,
}
impl TryResolveChallenge<ChallengeConfigGroup> for ChallengeConfigGroupData {
    fn try_resolve(
        self,
        name: String,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeConfigGroup, ChallengeParseError> {
        Ok(ChallengeConfigGroup {
            name,
            parts: self
                .parts
                .into_iter()
                .map(|(part_name, part_data)| {
                    part_data.try_resolve(
                        part_name,
                        self.command.clone().or(inherit_command.clone()),
                        inherit_config.merge(&self.config),
                    )
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct ChallengeConfigPartData {
    command: Option<ChallengeCommandData>,
    #[serde(flatten)]
    config: CommandConfig,

    cases: HashMap<String, ChallengeCaseData>,
}
impl TryResolveChallenge<ChallengeConfigPart> for ChallengeConfigPartData {
    fn try_resolve(
        self,
        name: String,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeConfigPart, ChallengeParseError> {
        Ok(ChallengeConfigPart {
            name: name.clone(),
            command: self
                .command
                .or(inherit_command)
                .ok_or(ChallengeParseError::NoCommandFound(name.clone()))?
                .into(),
            cases: self
                .cases
                .into_iter()
                .map(|(case_name, case_data)| {
                    case_data.try_resolve(
                        case_name,
                        name.clone(),
                        inherit_config.merge(&self.config),
                    )
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ChallengeConfigNode {
    Group(ChallengeConfigGroupData),
    Part(ChallengeConfigPartData),
}
impl TryResolveChallenge<ChallengeConfig> for ChallengeConfigNode {
    fn try_resolve(
        self,
        name: String,
        inherit_command: Option<ChallengeCommandData>,
        inherit_config: CommandConfig,
    ) -> Result<ChallengeConfig, ChallengeParseError> {
        match self {
            ChallengeConfigNode::Group(group) => Ok(ChallengeConfig::Group(group.try_resolve(
                name,
                inherit_command,
                inherit_config,
            )?)),
            ChallengeConfigNode::Part(part) => Ok(ChallengeConfig::Part(part.try_resolve(
                name,
                inherit_command,
                inherit_config,
            )?)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChallengeConfigData {
    name: String,
    #[serde(flatten)]
    node: ChallengeConfigNode,
}
impl TryInto<ChallengeConfig> for ChallengeConfigData {
    type Error = ChallengeParseError;

    fn try_into(self) -> Result<ChallengeConfig, Self::Error> {
        self.node.try_resolve_default(self.name)
    }
}
