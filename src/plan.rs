use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use config::{Config, Mode};
use error::Result;
use steps::{BuildStepFactory, TestStepFactory};

pub fn run_tests(config: Config) {
    TestPlan::new(config).execute();
}

struct TestPlan {
    config: Config,
    steps: Vec<Box<TestStepFactory>>,
    crates: Vec<PathBuf>,
}

impl TestPlan {
    pub fn new(config: Config) -> Self {
        let crates = WalkDir::new(&config.base_dir)
            .max_depth(1)
            .into_iter()
            .map(|entry| entry.unwrap().path().into())
            .collect();

        let steps: Vec<Box<TestStepFactory>> = match config.mode {
            Mode::BuildFail => vec![],
            Mode::BuildSuccess => vec![Box::new(BuildStepFactory::new())],
            Mode::Expand => vec![Box::new(BuildStepFactory::new())],
        };

        TestPlan {
            config,
            crates,
            steps,
        }
    }

    pub fn execute(self) {
        for crate_path in &self.crates {
            self.execute_steps(&crate_path).unwrap(); // TODO: error handling
        }
    }

    fn execute_steps(&self, crate_path: &Path) -> Result<()> {
        let local_steps: Vec<_> = self.steps
            .iter()
            .map(|factory| factory.initialize(&self.config, crate_path))
            .collect();

        for step in local_steps {
            step?.execute(&self.config, &PathBuf::from("/tmp"))?; // TODO
        }

        Ok(())
    }
}
