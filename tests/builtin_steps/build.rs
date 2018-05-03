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
        format!(
            "Unable to build the crate!
### stdout:

### stderr:
   Compiling fail-1 v0.1.0 (file://{})
error[E0432]: unresolved import `mod2::func3`
 --> src/lib.rs:2:9
  |
2 |     use mod2::func3; //~  WARNING another warning
  |         ^^^^^^^^^^^ no `func3` in `mod2`. Did you mean to use `func2`?

error[E0412]: cannot find type `NonExistingType` in this scope
  --> src/lib.rs:12:19
   |
12 |     fn func2() -> NonExistingType {{
   |                   ^^^^^^^^^^^^^^^ not found in this scope

error: aborting due to 2 previous errors

Some errors occurred: E0412, E0432.
For more information about an error, try `rustc --explain E0412`.
error: Could not compile `fail-1`.

To learn more, run the command again with --verbose.

###",
            crate_path.to_string_lossy()
        )
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
