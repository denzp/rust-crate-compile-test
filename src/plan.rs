use std::io::{stderr, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tempfile::tempdir;
use walkdir::WalkDir;

use config::{Config, Mode};
use error::Result;
use formatting;
use steps::{build::BuildStepFactory, check_errors::CheckErrorsStepFactory, TestStepFactory};

lazy_static! {
    static ref TESTING_MUTEX: Mutex<()> = Mutex::new(());
}

pub fn run_tests(config: Config) {
    if let Err(error) = run_tests_with_writer(config, stderr()) {
        panic!("{}", error);
    }
}

pub fn run_tests_with_writer<W: Write>(config: Config, writer: W) -> Result<()> {
    TestPlan::new(config).execute(writer)
}

struct TestPlan {
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

    pub fn execute<W: Write>(self, mut writer: W) -> Result<()> {
        let _lock = TESTING_MUTEX.lock().unwrap();

        let mut successful: usize = 0;
        let mut failed: usize = 0;

        let results: Vec<_> = self.crates
            .iter()
            .map(|ref crate_path| match self.execute_steps(&crate_path) {
                Ok(result) => {
                    writeln!(
                        writer,
                        "testing crate {} ... OK",
                        crate_path.to_string_lossy()
                    )?;

                    successful += 1;
                    Ok(result)
                }

                Err(error) => {
                    writeln!(
                        writer,
                        "testing crate {} ... FAILED",
                        crate_path.to_string_lossy()
                    )?;

                    failed += 1;
                    Err(error)
                }
            })
            .collect();

        for result in self.crates.iter().zip(results) {
            if let Err(error) = result.1 {
                writeln!(writer, "\n{}:", result.0.to_string_lossy())?;

                writeln!(
                    writer,
                    "{}",
                    formatting::prefix_each_line(error.to_string(), "  ")
                )?;
            }
        }

        if failed > 0 {
            bail!("{} tests failed", failed);
        } else {
            Ok(())
        }
    }

    fn execute_steps(&self, crate_path: &Path) -> Result<()> {
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
