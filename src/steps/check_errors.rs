use failure::ResultExt;
use regex::Regex;
use serde_json as json;
use std::cmp;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{TestStep, TestStepFactory};
use cargo_messages;
use config::{Config, Profile};
use error::{Result, TestingError};
use utils::SourceCodeAnalyser;

pub use cargo_messages::DiagnosticLevel;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MessageLocation {
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum MessageType {
    None,
    Text(String),
    Regex(Regex),
}

#[derive(Debug, Clone)]
pub struct CompilerMessage {
    pub message: MessageType,
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub location: Option<MessageLocation>,
}

pub struct CheckErrorsStepFactory;

struct CheckErrorsStep {
    crate_dir: PathBuf,
    expected_messages: Vec<CompilerMessage>,
}

impl CheckErrorsStepFactory {
    pub fn new() -> Self {
        CheckErrorsStepFactory {}
    }
}

impl SourceCodeAnalyser<CompilerMessage> for CheckErrorsStepFactory {
    fn analyse_source_line(
        previous: &[CompilerMessage],
        path: &Path,
        line: (usize, &str),
    ) -> Result<Option<CompilerMessage>> {
        lazy_static! {
            static ref ERR_CODE_REGEX: Regex = Regex::new(r"^ *E\d{4} *$").unwrap();
            static ref MESSAGE_REGEX: Regex =
                Regex::new(r"// *~([\^]+|[\|])? +(ERROR|WARNING|NOTE|HELP) +(.+)").unwrap();
            static ref GLOBAL_MESSAGE_REGEX: Regex =
                Regex::new(r"// *~ +GLOBAL-(ERROR|WARNING|NOTE|HELP)-REGEX +(.+)").unwrap();
        }

        if let Some(captures) = GLOBAL_MESSAGE_REGEX.captures(line.1) {
            let message = CompilerMessage {
                message: MessageType::Regex(Regex::new(captures[2].trim()).unwrap()),

                code: None,
                location: None,
                level: captures[1].into(),
            };

            return Ok(Some(message));
        }

        if let Some(captures) = MESSAGE_REGEX.captures(line.1) {
            let location = match captures.get(1).map(|item| item.as_str()) {
                Some("|") => previous
                    .iter()
                    .last()
                    .and_then(|item| item.location.clone()),

                None => Some(MessageLocation {
                    file: path.into(),
                    line: line.0,
                }),

                relative @ _ => Some(MessageLocation {
                    file: path.into(),
                    line: line.0 - relative.unwrap().len(),
                }),
            };

            let (message, code) = match ERR_CODE_REGEX.is_match(&captures[3]) {
                true => (None, Some(captures[3].trim().into())),
                false => (Some(captures[3].trim().into()), None),
            };

            let message = CompilerMessage {
                message: message
                    .map(|item| MessageType::Text(item))
                    .unwrap_or(MessageType::None),

                code,
                location,
                level: captures[2].into(),
            };

            Ok(Some(message))
        } else {
            Ok(None)
        }
    }
}

impl CheckErrorsStep {
    pub fn new(crate_dir: PathBuf, expected_messages: Vec<CompilerMessage>) -> Self {
        CheckErrorsStep {
            crate_dir,
            expected_messages,
        }
    }

    fn find_actual_messages(&self, config: &Config, path: &Path) -> Result<Vec<CompilerMessage>> {
        let mut command = Command::new(&config.cargo_command);

        command.current_dir(&self.crate_dir);
        command.env("CARGO_TARGET_DIR", path);
        command.args(&["build", "--message-format", "json"]);

        if let Some(target) = config.target.as_ref() {
            command.args(&["--target", target]);
        }

        if config.profile == Profile::Release {
            command.arg("--release");
        }

        for (key, value) in &config.cargo_env {
            command.env(key, value);
        }

        let mut actual_messages = vec![];

        let raw_output = command.output()?;
        let stderr = String::from_utf8_lossy(&raw_output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&raw_output.stdout).into_owned();

        for line in stdout.lines() {
            let message = {
                json::from_str::<cargo_messages::Diagnostic>(line)
                    .context("Unable to parse Cargo JSON output")?
            };

            match (message.reason.as_str(), message.message) {
                ("compiler-message", Some(message)) => {
                    if message.spans.len() == 0 {
                        for child in &message.children {
                            actual_messages.push(child.clone().into());
                        }
                    }

                    if !message.message.starts_with("aborting")
                        && message.level != DiagnosticLevel::Empty
                    {
                        actual_messages.push(message.into());
                    }
                }

                _ => {}
            };
        }

        match raw_output.status.success() {
            false => {
                if actual_messages.len() > 0 {
                    Ok(actual_messages)
                } else {
                    bail!(TestingError::CrateBuildFailed { stdout, stderr })
                }
            }

            true => bail!(TestingError::UnexpectedBuildSuccess),
        }
    }
}

impl TestStepFactory for CheckErrorsStepFactory {
    fn initialize(&self, _config: &Config, crate_path: &Path) -> Result<Box<TestStep>> {
        Ok(Box::new(CheckErrorsStep::new(
            crate_path.into(),
            Self::analyse_crate(crate_path)?,
        )))
    }
}

impl TestStep for CheckErrorsStep {
    fn execute(&self, config: &Config, build_path: &Path) -> Result<()> {
        let actual_messages = self.find_actual_messages(config, build_path)?;

        let unexpected_messages: Vec<_> = actual_messages
            .clone()
            .into_iter()
            .filter(|item| !self.expected_messages.contains(item))
            .collect();

        let missing_messages: Vec<_> = self.expected_messages
            .clone()
            .into_iter()
            .filter(|item| !actual_messages.contains(item))
            .collect();

        if unexpected_messages.len() > 0 || missing_messages.len() > 0 {
            bail!(TestingError::MessageExpectationsFailed {
                unexpected: unexpected_messages,
                missing: missing_messages,
            });
        }

        Ok(())
    }
}

impl cmp::PartialEq for CompilerMessage {
    fn eq(&self, other: &CompilerMessage) -> bool {
        if self.location != other.location || self.level != other.level {
            return false;
        }

        if self.code.is_some() && other.code.is_some() {
            return self.code.as_ref().unwrap() == other.code.as_ref().unwrap();
        }

        match (&self.message, &other.message) {
            (MessageType::Text(ref lhs), MessageType::Text(ref rhs)) => lhs == rhs,

            (MessageType::Text(ref lhs), MessageType::Regex(ref rhs)) => rhs.is_match(lhs),
            (MessageType::Regex(ref lhs), MessageType::Text(ref rhs)) => lhs.is_match(rhs),

            (MessageType::Regex(ref lhs), MessageType::Regex(ref rhs)) => {
                lhs.as_str() == rhs.as_str()
            }

            _ => false,
        }
    }
}

impl fmt::Display for CompilerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.location {
            Some(ref location) => {
                writeln!(
                    f,
                    "file:    {}:{}",
                    &location.file.to_string_lossy(),
                    location.line
                )?;
            }

            None => {
                writeln!(f, "file:    none",)?;
            }
        };

        f.write_str("message: ")?;
        f.write_str(&match self.code {
            Some(ref code) => format!("({:?} {}) ", self.level, code),
            None => format!("({:?}) ", self.level),
        })?;

        match self.message {
            MessageType::Text(ref message) => write!(f, "{}", message)?,
            MessageType::Regex(ref expr) => write!(f, "Regex({})", expr.as_str())?,

            _ => {}
        }

        Ok(())
    }
}
