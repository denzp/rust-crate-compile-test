use std::path::{Path, PathBuf};

#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate failure;
extern crate colored;

#[macro_use]
mod utils;

extern crate crate_compile_test;
use crate_compile_test::prelude::*;
use crate_compile_test::steps::{TestStep, TestStepFactory};

#[test]
fn it_should_report_failure() {
    colored::control::set_override(false);

    let mut actual_output_bytes: Vec<u8> = Vec::new();

    let result = {
        let mut runner = TestRunner::new(&mut actual_output_bytes);

        runner.add("failure", || {
            Config::new(Mode::BuildFail, "example/tests/build-fail")
        });

        runner.start().unwrap()
    };

    assert_eq!(result.is_success(), false);
    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.failure.output")
    );
}

#[test]
fn it_should_report_about_unexpected_success() {
    colored::control::set_override(false);

    let mut actual_output_bytes: Vec<u8> = Vec::new();

    let result = {
        let mut runner = TestRunner::new(&mut actual_output_bytes);

        runner.add("unexpected success", || {
            Config::new(Mode::BuildFail, "example/tests/build-success")
        });

        runner.start().unwrap()
    };

    assert_eq!(result.is_success(), false);
    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.unexpected_success.output")
    );
}

#[test]
fn it_should_report_success() {
    colored::control::set_override(false);

    let mut actual_output_bytes: Vec<u8> = Vec::new();

    let result = {
        let mut runner = TestRunner::new(&mut actual_output_bytes);

        runner.add("success", || {
            Config::new(Mode::BuildSuccess, "example/tests/build-success")
        });

        runner.start().unwrap()
    };

    assert_eq!(result.is_success(), true);
    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.success.output")
    );
}

#[test]
fn it_should_report_unexpected_failure() {
    colored::control::set_override(false);

    let mut actual_output_bytes: Vec<u8> = Vec::new();

    let result = {
        let mut runner = TestRunner::new(&mut actual_output_bytes);

        runner.add("unexpected failure", || {
            let mut config = Config::new(Mode::BuildSuccess, "example/tests/build-fail");

            config.crates_filter =
                Box::new(|path| path != Path::new("example/tests/build-fail/fail-4"));

            config
        });

        runner.start().unwrap()
    };

    assert_eq!(result.is_success(), false);
    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.unexpected_failure.output")
    );
}

#[test]
fn it_should_use_crates_filter() {
    colored::control::set_override(false);

    let mut actual_output_bytes: Vec<u8> = Vec::new();

    let result = {
        let mut runner = TestRunner::new(&mut actual_output_bytes);

        runner.add("filters", || {
            let mut config = Config::new(Mode::BuildSuccess, "example/tests/build-success");

            config.crates_filter =
                Box::new(|path| path != Path::new("example/tests/build-success/success-1"));

            config
        });

        runner.start().unwrap()
    };

    assert_eq!(result.is_success(), true);
    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.filter.output")
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
    colored::control::set_override(false);

    let mut actual_output_bytes: Vec<u8> = Vec::new();

    let result = {
        let mut runner = TestRunner::new(&mut actual_output_bytes);

        runner.add("additional steps", || {
            let mut config = Config::new(Mode::BuildSuccess, "example/tests/build-success");

            config
                .additional_steps
                .push(Box::new(DummyTestStepFactory {}));

            config
        });

        runner.start().unwrap()
    };

    assert_eq!(result.is_success(), false);
    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.additional_steps.output")
    );
}

#[test]
fn it_should_report_multiple_tests() {
    colored::control::set_override(false);

    let mut actual_output_bytes: Vec<u8> = Vec::new();

    let result = {
        let mut runner = TestRunner::new(&mut actual_output_bytes);

        runner.add("unexpected success", || {
            Config::new(Mode::BuildFail, "example/tests/build-success")
        });

        runner.add("failure", || {
            Config::new(Mode::BuildFail, "example/tests/build-fail")
        });

        runner.start().unwrap()
    };

    assert_eq!(result.is_success(), false);
    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.multiple.output")
    );
}
