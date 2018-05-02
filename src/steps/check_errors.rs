use failure::ResultExt;
use serde_json as json;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{TestStep, TestStepFactory};
use cargo_messages::{self, DiagnosticLevel};
use config::Config;
use error::{Result, TestingError};

#[derive(Debug, PartialEq, Deserialize)]
pub struct MessageLocation {
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Deserialize)]
pub struct CompilerMessage {
    pub message: String,
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub location: MessageLocation,
}

pub struct CheckErrorsStepFactory;
pub struct CheckErrorsStep {
    crate_dir: PathBuf,
}

impl CheckErrorsStepFactory {
    pub fn new() -> Self {
        CheckErrorsStepFactory {}
    }
}

impl CheckErrorsStep {
    pub fn new(crate_dir: PathBuf) -> Self {
        CheckErrorsStep { crate_dir }
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
        Ok(Box::new(CheckErrorsStep::new(crate_path.into())))
    }
}

impl TestStep for CheckErrorsStep {
    fn execute(&self, config: &Config, build_path: &Path) -> Result<()> {
        let actual_messages = self.find_actual_messages(config, build_path)?;

        if actual_messages.len() > 0 {
            bail!(TestingError::MessageExpectationsFailed {
                unexpected: actual_messages,
                missing: vec![],
            })
        }

        Ok(())
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

        f.write_str(&self.message)?;

        Ok(())
    }
}
