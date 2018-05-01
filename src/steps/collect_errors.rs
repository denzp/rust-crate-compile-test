use failure::ResultExt;
use serde_json as json;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{TestStep, TestStepFactory};
use config::Config;
use error::{Result, TestingError};

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,

    #[serde(rename = "")]
    Empty,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct MessageLocation {
    file: PathBuf,
    line: usize,
}

#[derive(Debug, Deserialize)]
pub struct CompilerMessage {
    message: String,
    level: DiagnosticLevel,
    code: Option<String>,
    location: MessageLocation,
}

pub struct CollectErrorsStepFactory;
pub struct CollectErrorsStep {
    crate_dir: PathBuf,
}

impl CollectErrorsStepFactory {
    pub fn new() -> Self {
        CollectErrorsStepFactory {}
    }
}

impl CollectErrorsStep {
    pub fn new(crate_dir: PathBuf) -> Self {
        CollectErrorsStep { crate_dir }
    }
}

impl TestStepFactory for CollectErrorsStepFactory {
    fn initialize(&self, _config: &Config, crate_path: &Path) -> Result<Box<TestStep>> {
        Ok(Box::new(CollectErrorsStep::new(crate_path.into())))
    }
}

impl TestStep for CollectErrorsStep {
    fn execute(&self, config: &Config, build_path: &Path) -> Result<()> {
        let mut command = Command::new(&config.cargo_command);

        command.current_dir(&self.crate_dir);
        command.env("CARGO_TARGET_DIR", build_path);
        command.args(&["build", "--message-format", "json"]);

        if let Some(target) = config.target.as_ref() {
            command.args(&["--target", target]);
        }

        for (key, value) in &config.cargo_env {
            command.env(key, value);
        }

        let raw_output = command.output()?;

        let mut actual_messages = vec![];

        let stderr = String::from_utf8_lossy(&raw_output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&raw_output.stdout).into_owned();
        {
            let stdout_lines = stdout.lines();

            for line in stdout_lines {
                let message = {
                    json::from_str::<cargo_messages_json::Diagnostic>(line)
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
        }

        match raw_output.status.success() {
            false => {
                if actual_messages.len() > 0 {
                    bail!(TestingError::MessageExpectationsFailed {
                        unexpected: actual_messages,
                        missing: vec![],
                    })
                } else {
                    bail!(TestingError::CrateBuildFailed { stdout, stderr })
                }
            }

            true => bail!(TestingError::UnexpectedBuildSuccess),
        }
    }
}

mod cargo_messages_json {
    use super::{CompilerMessage, DiagnosticLevel, MessageLocation};
    use std::path::PathBuf;

    #[derive(Debug, Deserialize)]
    pub struct Diagnostic {
        pub message: Option<DiagnosticMessage>,
        pub reason: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct DiagnosticMessage {
        pub message: String,
        pub level: DiagnosticLevel,
        pub code: Option<DiagnosticCode>,
        pub spans: Vec<DiagnosticSpan>,
    }

    #[derive(Debug, Deserialize)]
    pub struct DiagnosticSpan {
        pub file_name: String,
        pub line_start: usize,
        pub is_primary: bool,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct DiagnosticCode {
        pub code: String,
    }

    impl Default for DiagnosticSpan {
        fn default() -> Self {
            DiagnosticSpan {
                file_name: "unknown".into(),
                line_start: 1,
                is_primary: true,
            }
        }
    }

    impl Into<CompilerMessage> for DiagnosticMessage {
        fn into(self) -> CompilerMessage {
            let span = self.spans
                .into_iter()
                .filter(|item| item.is_primary)
                .nth(0)
                .unwrap_or_default();

            CompilerMessage {
                message: self.message,
                level: self.level,
                code: self.code.map(|item| item.code),
                location: MessageLocation {
                    file: PathBuf::from(span.file_name),
                    line: span.line_start,
                },
            }
        }
    }
}
