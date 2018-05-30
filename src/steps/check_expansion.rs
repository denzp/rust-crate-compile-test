use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use super::{TestStep, TestStepFactory};
use config::{Config, Profile};
use error::{Result, TestingError};
use utils::SourceCodeAnalyser;

#[derive(Debug, PartialEq)]
pub struct ExpectedExpansion {
    pub module: String,
    pub expansion: String,
}

pub struct CheckExpansionStepFactory;

pub struct CheckExpansionStep {
    crate_dir: PathBuf,
}

impl CheckExpansionStepFactory {
    pub fn new() -> Self {
        CheckExpansionStepFactory {}
    }

    fn transform_path_into_module(path: &Path) -> String {
        lazy_static! {
            static ref MODULE_NAME_REGEX: Regex = Regex::new(r"src/(.+?)(/mod)?.rs").unwrap();
        }

        match MODULE_NAME_REGEX.captures(&path.to_string_lossy()) {
            Some(captures) => captures[1].into(),
            None => path.to_string_lossy().into(),
        }
    }
}

impl CheckExpansionStep {
    pub fn new(crate_dir: PathBuf, _expected_expansions: Vec<ExpectedExpansion>) -> Self {
        CheckExpansionStep { crate_dir }
    }

    pub fn find_actual_expansion(
        config: &Config,
        crate_path: &Path,
        build_path: &Path,
    ) -> Result<BTreeMap<String, String>> {
        let mut command = Command::new(&config.cargo_command);

        command.current_dir(&crate_path);
        command.env("CARGO_TARGET_DIR", build_path);
        command.arg("rustc");

        if let Some(target) = config.target.as_ref() {
            command.args(&["--target", target]);
        }

        if config.profile == Profile::Release {
            command.arg("--release");
        }

        command.args(&["--", "-Zunpretty=hir"]);

        for (key, value) in &config.cargo_env {
            command.env(key, value);
        }

        let raw_output = command.output()?;
        let stderr = String::from_utf8_lossy(&raw_output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&raw_output.stdout).into_owned();

        match raw_output.status.success() {
            false => bail!(TestingError::CrateBuildFailed { stdout, stderr }),
            true => Self::analyse_actual_expansion(&stdout),
        }
    }

    pub fn analyse_actual_expansion(_output: &str) -> Result<BTreeMap<String, String>> {
        // println!("{}", output);

        Ok(BTreeMap::new())
    }
}

impl SourceCodeAnalyser<ExpectedExpansion> for CheckExpansionStepFactory {
    fn analyse_source_line(
        _previous: &[ExpectedExpansion],
        path: &Path,
        line: (usize, &str),
    ) -> Result<Option<ExpectedExpansion>> {
        lazy_static! {
            static ref EXPANSION_REGEX: Regex = Regex::new(r"// *~ +EXPAND +(.+)").unwrap();
        }

        if let Some(captures) = EXPANSION_REGEX.captures(line.1) {
            Ok(Some(ExpectedExpansion {
                module: Self::transform_path_into_module(path),
                expansion: captures[1].into(),
            }))
        } else {
            Ok(None)
        }
    }
}

impl TestStepFactory for CheckExpansionStepFactory {
    fn initialize(&self, _config: &Config, crate_path: &Path) -> Result<Box<TestStep>> {
        Ok(Box::new(CheckExpansionStep::new(
            crate_path.into(),
            Self::analyse_crate(crate_path)?,
        )))
    }
}

impl TestStep for CheckExpansionStep {
    fn execute(&self, config: &Config, build_path: &Path) -> Result<()> {
        Self::find_actual_expansion(config, &self.crate_dir, build_path)?;

        Ok(())
    }
}
