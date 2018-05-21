use std::path::{Path, PathBuf};
use tempfile::tempdir;
use walkdir::WalkDir;

use config::{Config, Mode};
use error::Result;
use steps::{build::BuildStepFactory, check_errors::CheckErrorsStepFactory, TestStepFactory};

pub struct TestPlan {
    config: Config,
    steps: Vec<Box<TestStepFactory>>,
    crates: Vec<PathBuf>,
}

impl TestPlan {
    pub fn new(mut config: Config) -> Self {
        let crates = WalkDir::new(&config.base_dir)
            .max_depth(1)
            .min_depth(1)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path().into())
            .collect();

        let mut steps: Vec<Box<TestStepFactory>> = match config.mode {
            Mode::BuildFail => vec![Box::new(CheckErrorsStepFactory::new())],
            Mode::BuildSuccess => vec![Box::new(BuildStepFactory::new())],
            Mode::Expand => vec![Box::new(BuildStepFactory::new())],
        };

        if config.additional_steps.len() > 0 {
            steps.append(&mut config.additional_steps);
        }

        TestPlan {
            config,
            crates,
            steps,
        }
    }

    pub fn crates(&self) -> &[PathBuf] {
        &self.crates
    }

    pub fn is_crate_filtered_out(&self, crate_path: &Path) -> bool {
        (self.config.crates_filter)(crate_path) == false
    }

    pub fn execute_steps(&self, crate_path: &Path) -> Result<()> {
        let build_path = tempdir()?;

        let local_steps: Vec<_> = self.steps
            .iter()
            .map(|factory| factory.initialize(&self.config, crate_path))
            .collect();

        for step in local_steps {
            step?.execute(&self.config, build_path.as_ref())?;
        }

        Ok(())
    }
}
