use std::env;
use std::fs::metadata;
use std::path::Path;
use tempfile::tempdir;

use crate_compile_test::config::{Config, Mode};
use crate_compile_test::steps::{BuildStepFactory, TestStepFactory};

#[test]
fn it_should_handle_success() {
    let step = BuildStepFactory::new();
    let output_path = tempdir().unwrap();

    let config = Config::new(Mode::BuildSuccess, "example-tests/build-success");

    step.initialize(&config, &Path::new("example-tests/build-success/success-1"))
        .unwrap()
        .execute(&config, output_path.as_ref())
        .expect("It should successfully build the crate");

    assert!(
        metadata(output_path.as_ref().join("debug/libsuccess_1.rlib"))
            .expect("Build output should exist")
            .is_file()
    );
}

#[test]
fn it_should_handle_fail() {
    let step = BuildStepFactory::new();

    let output_path = tempdir().unwrap();
    let mut crate_path = env::current_dir().unwrap();
    crate_path.push(Path::new("example-tests/build-fail/fail-1"));

    let config = Config::new(Mode::BuildSuccess, "example-tests/build-fail");

    let error = {
        step.initialize(&config, &crate_path)
            .unwrap()
            .execute(&config, output_path.as_ref())
            .expect_err("It should fail building the crate")
    };

    assert_eq!(
        error.to_string(),
        read_output!("tests/ui/fail-1.build.output")
            .replace("$CRATE_PATH", &crate_path.to_string_lossy())
    );
}

#[test]
fn it_should_use_cargo_from_config() {
    let step = BuildStepFactory::new();

    let mut config = Config::new(Mode::BuildSuccess, "example-tests/build-success");
    config.cargo_command = "some-non-existing-command".into();

    let output_path = tempdir().unwrap();

    step.initialize(&config, &Path::new("example-tests/build-success/success-1"))
        .unwrap()
        .execute(&config, output_path.as_ref())
        .expect_err("It should fail");
}

#[test]
fn it_should_use_cargo_env_from_config() {
    let step = BuildStepFactory::new();
    let output_path = tempdir().unwrap();

    let mut config = Config::new(Mode::BuildSuccess, "example-tests/build-success");
    config.add_cargo_env("RUSTFLAGS", "--non-existing-flag");

    let error = {
        step.initialize(&config, &Path::new("example-tests/build-success/success-1"))
            .unwrap()
            .execute(&config, output_path.as_ref())
            .expect_err("It should fail")
    };

    assert!(
        error
            .to_string()
            .contains("Unrecognized option: \'non-existing-flag\'")
    );
}

#[test]
fn it_should_use_target_from_config() {
    let step = BuildStepFactory::new();
    let output_path = tempdir().unwrap();

    let mut config = Config::new(Mode::BuildSuccess, "example-tests/build-success");
    config.target = Some("non-existing-target".into());

    let error = {
        step.initialize(&config, &Path::new("example-tests/build-success/success-1"))
            .unwrap()
            .execute(&config, output_path.as_ref())
            .expect_err("It should fail")
    };

    assert!(
        error
            .to_string()
            .contains("Could not find specification for target \"non-existing-target\"")
    );
}
