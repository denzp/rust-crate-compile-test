use std::path::PathBuf;

use steps::{CompilerMessage, MessageLocation};

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,

    #[serde(rename = "")]
    Empty,
}

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
