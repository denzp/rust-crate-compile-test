use failure::ResultExt;
use regex::Regex;
use serde_json as json;
use std::cmp;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use super::{TestStep, TestStepFactory};
use cargo_messages;
use config::Config;
use error::{Result, TestingError};

pub use cargo_messages::DiagnosticLevel;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MessageLocation {
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompilerMessage {
    pub message: Option<String>,
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub location: MessageLocation,
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

    pub fn collect_crate_messages(crate_path: &Path) -> Result<Vec<CompilerMessage>> {
        let sources = WalkDir::new(&crate_path.join("src"))
            .into_iter()
            .map(|entry| entry.unwrap())
            .filter_map(
                |entry| match entry.path().extension().and_then(|item| item.to_str()) {
                    Some("rs") => Some(PathBuf::from(entry.path())),
                    _ => None,
                },
            );

        let mut messages = vec![];

        for path in sources {
            let source_path = path.strip_prefix(crate_path)?;
            let source_file = BufReader::new({
                File::open(&path).context(format!("Unable to open source at {:?}", path))?
            });

            source_file.lines().fold(1, |line_num, line| {
                Self::analyse_source_line(&source_path, (line_num, &line.unwrap()), &mut messages);

                line_num + 1
            });
        }

        Ok(messages)
    }

    fn analyse_source_line(path: &Path, line: (usize, &str), messages: &mut Vec<CompilerMessage>) {
        lazy_static! {
            static ref ERR_CODE_REGEX: Regex = Regex::new(r"^ *E\d{4} *$").unwrap();
            static ref MESSAGE_REGEX: Regex =
                Regex::new(r"// *~([\^]+|[\|])? +(ERROR|WARNING|NOTE|HELP) +(.+)").unwrap();
        }

        if let Some(captures) = MESSAGE_REGEX.captures(line.1) {
            let location = match captures.get(1).map(|item| item.as_str()) {
                Some("|") => MessageLocation {
                    file: path.into(),
                    line: messages
                        .iter()
                        .last()
                        .map(|item| item.location.line)
                        .unwrap_or(1),
                },

                None => MessageLocation {
                    file: path.into(),
                    line: line.0,
                },

                relative @ _ => MessageLocation {
                    file: path.into(),
                    line: line.0 - relative.unwrap().len(),
                },
            };

            let (message, code) = match ERR_CODE_REGEX.is_match(&captures[3]) {
                true => (None, Some(captures[3].trim().into())),
                false => (Some(captures[3].trim().into()), None),
            };

            let message = CompilerMessage {
                message,
                code,

                location,
                level: captures[2].into(),
            };

            messages.push(message);
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
                    if message.spans.len() > 0 && message.level != DiagnosticLevel::Empty {
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
            Self::collect_crate_messages(crate_path)?,
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
            (Some(ref lhs), Some(ref rhs)) => lhs == rhs,

            _ => false,
        }
    }
}

impl fmt::Display for CompilerMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!(
            "file:    {}:{}\n",
            &self.location.file.to_string_lossy(),
            self.location.line
        ))?;

        f.write_str("message: ")?;
        f.write_str(&match self.code {
            Some(ref code) => format!("({:?} {}) ", self.level, code),
            None => format!("({:?}) ", self.level),
        })?;

        if let Some(ref message) = self.message {
            f.write_str(&message)?;
        }

        Ok(())
    }
}
