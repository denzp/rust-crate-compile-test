use std::path::{Path, PathBuf};
use std::process::Command;

use super::{TestStep, TestStepFactory};
use config::Config;
use error::{Result, TestingError};

pub struct BuildStepFactory;

struct BuildStep {
    crate_dir: PathBuf,
}

impl BuildStepFactory {
    pub fn new() -> Self {
        BuildStepFactory {}
    }
}

impl BuildStep {
    pub fn new(crate_dir: PathBuf) -> Self {
        BuildStep { crate_dir }
    }
}

impl TestStepFactory for BuildStepFactory {
    fn initialize(&self, _config: &Config, crate_path: &Path) -> Result<Box<TestStep>> {
        Ok(Box::new(BuildStep::new(crate_path.into())))
    }
}

impl TestStep for BuildStep {
    fn execute(&self, config: &Config, build_path: &Path) -> Result<()> {
        let mut command = Command::new(&config.cargo_command);

        command.current_dir(&self.crate_dir);
        command.env("CARGO_TARGET_DIR", build_path);
        command.arg("build");

        if let Some(target) = config.target.as_ref() {
            command.args(&["--target", target]);
        }

        for (key, value) in &config.cargo_env {
            command.env(key, value);
        }

        let raw_output = command.output()?;
        let stdout = String::from_utf8_lossy(&raw_output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&raw_output.stderr).into_owned();

        match raw_output.status.success() {
            false => bail!(TestingError::CrateBuildFailed { stdout, stderr }),
            true => Ok(()),
        }
    }
}
