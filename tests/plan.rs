use std::path::{Path, PathBuf};

#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate failure;

#[macro_use]
mod utils;

extern crate crate_compile_test;
use crate_compile_test::prelude::*;
use crate_compile_test::steps::{TestStep, TestStepFactory};

#[test]
fn it_should_report_failure() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();
    let config = Config::new(Mode::BuildFail, "example/tests/build-fail");

    run_tests_with_writer(config, &mut actual_output_bytes).unwrap_err();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.failure.output")
    );
}

#[test]
fn it_should_report_about_unexpected_success() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();
    let config = Config::new(Mode::BuildFail, "example/tests/build-success");

    run_tests_with_writer(config, &mut actual_output_bytes).unwrap_err();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.unexpected_success.output")
    );
}

#[test]
fn it_should_report_success() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();
    let config = Config::new(Mode::BuildSuccess, "example/tests/build-success");

    run_tests_with_writer(config, &mut actual_output_bytes).unwrap();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.success.output")
    );
}

#[test]
fn it_should_report_unexpected_failure() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();
    let config = Config::new(Mode::BuildSuccess, "example/tests/build-fail");

    run_tests_with_writer(config, &mut actual_output_bytes).unwrap_err();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.unexpected_failure.output")
    );
}

struct DummyTestStepFactory;
struct DummyTestStep {
    crate_path: PathBuf,
}

impl TestStepFactory for DummyTestStepFactory {
    fn initialize(&self, _config: &Config, crate_path: &Path) -> Result<Box<TestStep>> {
        Ok(Box::new(DummyTestStep {
            crate_path: crate_path.into(),
        }))
    }
}

impl TestStep for DummyTestStep {
    fn execute(&self, _config: &Config, _build_path: &Path) -> Result<()> {
        bail!(
            "dummy additional step failed on crate `{}`",
            self.crate_path.to_string_lossy()
        )
    }
}

#[test]
fn it_should_run_additional_steps() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();
    let mut config = Config::new(Mode::BuildSuccess, "example/tests/build-success");

    config
        .additional_steps
        .push(Box::new(DummyTestStepFactory {}));

    run_tests_with_writer(config, &mut actual_output_bytes).unwrap_err();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.additional_steps.output")
    );
}
