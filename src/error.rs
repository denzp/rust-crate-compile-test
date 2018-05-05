use failure::Error;
use std::fmt;

use steps::CompilerMessage;

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
                header: "Expected the crate to build with error, but the build was succesful!",
                content: None,
            },

            TestingError::CrateBuildFailed { stdout, stderr } => ErrorDisplay {
                header: "Unable to build the crate!",
                content: Some(format!(
                    "### stdout:\n{}\n### stderr:\n{}\n###",
                    stdout, stderr
                )),
            },

            TestingError::MessageExpectationsFailed {
                unexpected,
                missing,
            } => ErrorDisplay {
                header: "Compiler messages don't fulfill expectations!",
                content: Some(format!(
                    "### Unexpected messages:\n{}\n### Missing messages:\n{}\n###",
                    utils::display_list(unexpected),
                    utils::display_list(missing)
                )),
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
            write!(f, "\n\n{}", content.as_ref())?;
        }

        Ok(())
    }
}

mod utils {
    use std::fmt;

    pub fn display_list<T: fmt::Display>(list: &Vec<T>) -> String {
        if list.len() == 0 {
            return "".into();
        }

        trim_lines(prefix_each_line(
            String::from("- ")
                + &list.iter()
                    .map(|item| prefix_each_next_line(item.to_string(), "  "))
                    .collect::<Vec<_>>()
                    .join("\n\n- ") + "\n",
            "  ",
        ))
    }

    fn trim_lines(input: String) -> String {
        input
            .lines()
            .map(|line| line.trim_right())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn prefix_each_line<S: AsRef<str>>(input: S, prefix: &str) -> String {
        String::from(prefix) + &prefix_each_next_line(input, prefix)
    }

    pub fn prefix_each_next_line<S: AsRef<str>>(input: S, prefix: &str) -> String {
        input.as_ref().replace("\n", &format!("\n{}", prefix))
    }
}
