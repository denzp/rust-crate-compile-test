use std::path::Path;

use config::Config;
use error::Result;

pub trait TestStepFactory {
    fn initialize(&self, config: &Config, crate_path: &Path) -> Result<Box<TestStep>>;
}

pub trait TestStep {
    fn execute(&self, config: &Config, build_path: &Path) -> Result<()>;
}

pub mod build;
pub mod check_errors;
