use failure::Error;

use steps::CompilerMessage;

#[derive(Debug, Fail)]
pub enum TestingError {
    #[fail(display = "Can't find a crate at: {}", path)]
    MissingCrate { path: String },

    #[fail(
        display = "Unable to build the crate!\n### stdout:\n{}\n### stderr:\n{}\n###",
        stdout,
        stderr
    )]
    CrateBuildFailed { stdout: String, stderr: String },

    #[fail(display = "Expected the crate to build with error, but the build was succesful!")]
    UnexpectedBuildSuccess,

    #[fail(
        display = "Compiler messages don't fulfill expectations!\n### Unexpected messages:\n{:#?}\n### Missing messages:\n{:#?}\n###",
        unexpected,
        missing
    )]
    MessageExpectationsFailed {
        unexpected: Vec<CompilerMessage>,
        missing: Vec<CompilerMessage>,
    },
}

pub type Result<T> = ::std::result::Result<T, Error>;
