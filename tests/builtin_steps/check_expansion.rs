use std::collections::BTreeMap;
use std::path::Path;
use tempfile::tempdir;

use crate_compile_test::config::{Config, Mode};
use crate_compile_test::steps::TestStepFactory;
use crate_compile_test::utils::SourceCodeAnalyser;

use crate_compile_test::steps::check_expansion::{
    CheckExpansionStep, CheckExpansionStepFactory, ExpectedExpansion,
};

#[test]
fn it_should_collect_expected_expansions() {
    let crate_path = Path::new("example/tests/expand/expand-1");
    let messages = CheckExpansionStepFactory::analyse_crate(&crate_path).unwrap();

    assert_eq!(
        messages,
        &[
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "x = 2;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "x = 1 + 1;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "y = 1 + 1;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "y = 2 + 2;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "y = 4 + 4;".into(),
            },
            ExpectedExpansion {
                module: "mod_2".into(),
                expansion: "pub fn other_fn(_arg: u32) -> u32 {".into(),
            },
            ExpectedExpansion {
                module: "mod_2".into(),
                expansion: "0".into(),
            },
            ExpectedExpansion {
                module: "mod_2".into(),
                expansion: "}".into(),
            },
            ExpectedExpansion {
                module: "mod_2/nested_mod_1".into(),
                expansion: "t = [4, 3, 2, 6];".into(),
            },
            ExpectedExpansion {
                module: "mod_1".into(),
                expansion: "pub fn div_n(a: f64) -> f64 {".into(),
            },
            ExpectedExpansion {
                module: "mod_1".into(),
                expansion: "a / 4 as f64".into(),
            },
            ExpectedExpansion {
                module: "mod_1".into(),
                expansion: "}".into(),
            },
        ]
    );
}

#[test]
fn it_should_collect_actual_expansion() {
    let output_path = tempdir().unwrap();
    let config = Config::new(Mode::BuildFail, "example/tests/expand");

    let actual_expansion = CheckExpansionStep::find_actual_expansion(
        &config,
        &Path::new("example/tests/expand/expand-1"),
        output_path.as_ref(),
    );

    let mut expected_expansion = BTreeMap::new();

    expected_expansion.insert(
        "lib".into(),
        String::from(
            r#"
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;

fn some_fn() {
    let x = 1 + 1;
    let mut y;
    y = 1 + 1;
    y = 1 + 2;
    y = 3 + 3;
    y = 4 + 4;
}
            "#.trim(),
        ),
    );

    expected_expansion.insert(
        "mod_1".into(),
        String::from(
            r#"
pub fn div_n(a: f64) -> f64 {
    a / 5 as f64
}
            "#.trim(),
        ),
    );

    expected_expansion.insert(
        "mod_2".into(),
        String::from(
            r#"
pub fn other_fn(_arg: u32) -> u32 {
    0
}
            "#.trim(),
        ),
    );

    expected_expansion.insert(
        "mod_2/nested_mod_1".into(),
        String::from(
            r#"
pub fn reverse() {
    let t = [4, 3, 2, 6];
}
            "#.trim(),
        ),
    );

    assert_eq!(actual_expansion.unwrap(), expected_expansion);
}

#[test]
fn it_should_use_cargo_from_config() {
    let step = CheckExpansionStepFactory::new();

    let mut config = Config::new(Mode::BuildFail, "example/tests/build-fail");
    config.cargo_command = "some-non-existing-command".into();

    let output_path = tempdir().unwrap();

    step.initialize(&config, &Path::new("example/tests/build-fail/fail-1"))
        .unwrap()
        .execute(&config, output_path.as_ref())
        .expect_err("It should fail");
}

#[test]
fn it_should_use_cargo_env_from_config() {
    let step = CheckExpansionStepFactory::new();
    let output_path = tempdir().unwrap();

    let mut config = Config::new(Mode::BuildFail, "example/tests/build-fail");
    config.add_cargo_env("RUSTFLAGS", "--non-existing-flag");

    let error = {
        step.initialize(&config, &Path::new("example/tests/build-fail/fail-1"))
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
    let step = CheckExpansionStepFactory::new();
    let output_path = tempdir().unwrap();

    let mut config = Config::new(Mode::BuildSuccess, "example/tests/build-success");
    config.target = Some("non-existing-target".into());

    let error = {
        step.initialize(&config, &Path::new("example/tests/build-fail/fail-1"))
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
