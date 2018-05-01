use failure::Error;

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
}

pub type Result<T> = ::std::result::Result<T, Error>;
