use std::io::Write;
use std::sync::Mutex;

use colored::*;
use failure::Error;

use config::Config;
use error::{Result, TestingError};
use plan::TestPlan;

pub struct TestRunner<'a> {
    tests: Vec<Test>,
    output: Mutex<&'a mut Write>,
}

pub struct TestResult {
    success: bool,
}

struct Test {
    name: &'static str,
    config: Box<Fn() -> Config>,
}

impl<'a> TestRunner<'a> {
    pub fn new(output: &'a mut Write) -> Self {
        TestRunner {
            output: Mutex::new(output),
            tests: vec![],
        }
    }

    pub fn add<F: Fn() -> Config + 'static>(&mut self, name: &'static str, config: F) {
        self.tests.push(Test {
            name,
            config: Box::new(config),
        });
    }

    pub fn start(self) -> Result<TestResult> {
        let mut overall_successful: usize = 0;
        let mut overall_failed: usize = 0;
        let mut overall_ignored: usize = 0;

        for test in &self.tests {
            writeln!(
                self.output.lock().unwrap(),
                "{}",
                format!(r#"Running "{}""#, test.name).underline()
            )?;

            let plan = TestPlan::new((test.config)());

            let mut successful: usize = 0;
            let mut failed: usize = 0;
            let mut ignored: usize = 0;

            let errors: Vec<Error> = plan.crates()
                .iter()
                .map(|crate_path| -> Result<()> {
                    if plan.is_crate_filtered_out(crate_path) {
                        writeln!(
                            self.output.lock().unwrap(),
                            "  testing crate {} ... {}",
                            crate_path.to_string_lossy().bold(),
                            "IGNORED".yellow(),
                        )?;

                        ignored += 1;
                        return Ok(());
                    }

                    match plan.execute_steps(&crate_path) {
                        Ok(()) => {
                            writeln!(
                                self.output.lock().unwrap(),
                                "  testing crate {} ... {}",
                                crate_path.to_string_lossy().bold(),
                                "OK".bright_green(),
                            )?;

                            successful += 1;
                            Ok(())
                        }

                        Err(error) => {
                            writeln!(
                                self.output.lock().unwrap(),
                                "  testing crate {} ... {}",
                                crate_path.to_string_lossy().bold(),
                                "FAILED".red()
                            )?;

                            failed += 1;
                            bail!(TestingError::TestFailed {
                                path: crate_path.clone(),
                                error,
                            });
                        }
                    }
                })
                .filter_map(|item| item.err())
                .collect();

            for error in errors {
                writeln!(self.output.lock().unwrap(), "\n{}", error)?;
            }

            writeln!(self.output.lock().unwrap(), "")?;

            overall_successful += successful;
            overall_failed += failed;
            overall_ignored += ignored;
        }

        writeln!(
            self.output.lock().unwrap(),
            "Summary: {} successful, {} failed, {} ignored.",
            overall_successful.to_string().bright_green(),
            overall_failed.to_string().red(),
            overall_ignored.to_string().yellow(),
        )?;

        Ok(TestResult {
            success: overall_failed == 0,
        })
    }
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        self.success
    }
}
