use std::fmt;
use std::path::PathBuf;

use colored::*;
use failure::Error;

use formatting;
use steps::check_errors::CompilerMessage;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum TestingError {
    UnexpectedBuildSuccess,

    CrateBuildFailed {
        stdout: String,
        stderr: String,
    },

    MessageExpectationsFailed {
        unexpected: Vec<CompilerMessage>,
        missing: Vec<CompilerMessage>,
    },

    TestFailed {
        path: PathBuf,
        error: Error,
    },

    UnableToParseExpansion,
}

struct ErrorDisplay<S1, S2>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    header: S1,
    content: Option<S2>,
}

impl fmt::Display for TestingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = match self {
            TestingError::UnexpectedBuildSuccess => ErrorDisplay {
                header: "Unexpectedly successful build!".into(),
                content: None,
            },

            TestingError::TestFailed { path, error } => ErrorDisplay {
                header: format!("{} failed:", path.to_string_lossy().bold().red()),
                content: Some(formatting::prefix_each_line(error.to_string(), "  ")),
            },

            TestingError::CrateBuildFailed { stdout, stderr } => ErrorDisplay {
                header: "Unable to build the crate!".into(),
                content: Some({
                    let mut output = String::new();

                    if stdout.len() > 0 {
                        output += &format!("\n{}", formatting::display_block("stdout", stdout));
                    }

                    if stderr.len() > 0 {
                        output += &format!("\n{}", formatting::display_block("stderr", stderr));
                    }

                    output
                }),
            },

            TestingError::MessageExpectationsFailed {
                unexpected,
                missing,
            } => ErrorDisplay {
                header: "Compiler messages don't fulfill expectations!".into(),
                content: Some(format!(
                    "\nUnexpected messages:\n{}\n\nMissing messages:\n{}",
                    formatting::display_list(unexpected),
                    formatting::display_list(missing)
                )),
            },

            TestingError::UnableToParseExpansion => ErrorDisplay {
                header: "Internal error: Unable to parse expanded output!".into(),
                content: None,
            },
        };

        display.fmt(f)
    }
}

impl<S1, S2> fmt::Display for ErrorDisplay<S1, S2>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.header.as_ref())?;

        if let Some(ref content) = self.content {
            write!(f, "\n{}", content.as_ref())?;
        }

        Ok(())
    }
}
