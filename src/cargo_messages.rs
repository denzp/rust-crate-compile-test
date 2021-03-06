use std::path::PathBuf;

use steps::check_errors::{CompilerMessage, MessageLocation, MessageType};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,

    #[serde(rename = "")]
    Empty,
}

#[derive(Debug, Deserialize)]
pub struct Diagnostic {
    pub message: Option<DiagnosticMessage>,
    pub reason: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DiagnosticMessage {
    pub message: String,
    pub level: DiagnosticLevel,
    pub code: Option<DiagnosticCode>,
    pub spans: Vec<DiagnosticSpan>,
    pub children: Vec<DiagnosticMessage>,
}

#[derive(Debug, Deserialize, Clone)]
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

impl<'a> From<&'a str> for DiagnosticLevel {
    fn from(text: &str) -> Self {
        match text {
            "ERROR" => DiagnosticLevel::Error,
            "WARNING" => DiagnosticLevel::Warning,
            "NOTE" => DiagnosticLevel::Note,
            "HELP" => DiagnosticLevel::Help,

            _ => DiagnosticLevel::Empty,
        }
    }
}

impl Into<CompilerMessage> for DiagnosticMessage {
    fn into(self) -> CompilerMessage {
        let location = self.spans
            .into_iter()
            .filter(|item| item.is_primary)
            .nth(0)
            .map(|span| MessageLocation {
                file: PathBuf::from(span.file_name),
                line: span.line_start,
            });

        CompilerMessage {
            message: MessageType::Text(self.message),
            level: self.level,
            code: self.code.map(|item| item.code),
            location,
        }
    }
}
