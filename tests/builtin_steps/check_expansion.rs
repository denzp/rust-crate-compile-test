use std::collections::BTreeMap;
use std::path::Path;
use tempfile::tempdir;

use crate_compile_test::config::{Config, Mode};
use crate_compile_test::steps::TestStepFactory;

use crate_compile_test::steps::check_expansion::{CheckExpansionStep, CheckExpansionStepFactory};

#[test]
fn it_should_tokenize_expressions() {
    assert_eq!(
        CheckExpansionStepFactory::parse_tokens(r#"any string"#.into()),
        to_owned_vec(&["any", "string"]),
    );

    assert_eq!(
        CheckExpansionStepFactory::parse_tokens(r#""any" string"#.into()),
        to_owned_vec(&[r#""any""#, "string"]),
    );

    assert_eq!(
        CheckExpansionStepFactory::parse_tokens(r#"a [..] b"#.into()),
        to_owned_vec(&["a", "[", "..", "]", "b"]),
    );

    assert_eq!(
        CheckExpansionStepFactory::parse_tokens(r#""a [..] b""#.into()),
        to_owned_vec(&[r#""a [..] b""#]),
    );
}

#[test]
fn it_should_collect_expected_expansions() {
    let crate_path = Path::new("example/tests/expand/expand-1");
    let actual_expectations = CheckExpansionStepFactory::find_expected_expansion(&crate_path);

    let mut expected_expectations = BTreeMap::new();

    expected_expectations.insert(
        "lib".into(),
        vec![
            to_owned_vec(&["x", "=", "2", ";"]),
            to_owned_vec(&["x", "=", "1", "+", "1", ";"]),
            to_owned_vec(&["y", "=", "1", "+", "1", ";"]),
            to_owned_vec(&["y", "=", "2", "+", "2", ";"]),
            to_owned_vec(&["y", "=", "4", "+", "4", ";"]),
        ],
    );

    expected_expectations.insert(
        "mod_2".into(),
        vec![
            to_owned_vec(&[
                "pub", "fn", "other_fn", "(", "_arg", ":", "u32", ")", "->", "u32", "{",
            ]),
            to_owned_vec(&["0"]),
            to_owned_vec(&["}"]),
        ],
    );

    expected_expectations.insert(
        "mod_2/nested_mod_1".into(),
        vec![to_owned_vec(&[
            "t", "=", "[", "4", ",", "3", ",", "2", ",", "6", "]", ";",
        ])],
    );

    expected_expectations.insert(
        "mod_1".into(),
        vec![
            to_owned_vec(&[
                "pub", "fn", "div_n", "(", "a", ":", "f64", ")", "->", "f64", "{",
            ]),
            to_owned_vec(&["a", "/", "4", "as", "f64"]),
            to_owned_vec(&["}"]),
        ],
    );

    assert_eq!(actual_expectations.unwrap(), expected_expectations);
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
        to_owned_vec(&[
            "#",
            "[",
            "prelude_import",
            "]",
            "use",
            "std",
            "::",
            "prelude",
            "::",
            "v1",
            "::",
            "*",
            ";",
            "#",
            "[",
            "macro_use",
            "]",
            "extern",
            "crate",
            "std",
            ";",
            "fn",
            "some_fn",
            "(",
            ")",
            "{",
            "let",
            "x",
            "=",
            "1",
            "+",
            "1",
            ";",
            "let",
            "mut",
            "y",
            ";",
            "y",
            "=",
            "1",
            "+",
            "1",
            ";",
            "y",
            "=",
            "1",
            "+",
            "2",
            ";",
            "y",
            "=",
            "3",
            "+",
            "3",
            ";",
            "y",
            "=",
            "4",
            "+",
            "4",
            ";",
            "}",
        ]),
    );

    expected_expansion.insert(
        "mod_1".into(),
        to_owned_vec(&[
            "pub", "fn", "div_n", "(", "a", ":", "f64", ")", "->", "f64", "{", "a", "/", "5", "as",
            "f64", "}",
        ]),
    );

    expected_expansion.insert(
        "mod_2".into(),
        to_owned_vec(&[
            "pub",
            "fn",
            "other_fn",
            "(",
            "_arg",
            ":",
            "u32",
            ")",
            "->",
            "u32",
            "{",
            "0",
            "}",
            "mod",
            "inner_mod",
            "{",
            "pub",
            "fn",
            "inner_fn",
            "(",
            "_arg",
            ":",
            "u32",
            ")",
            "->",
            "u32",
            "{",
            "1",
            "}",
            "}",
        ]),
    );

    expected_expansion.insert(
        "mod_2/nested_mod_1".into(),
        to_owned_vec(&[
            "pub", "fn", "reverse", "(", ")", "{", "let", "t", "=", "[", "4", ",", "3", ",", "2",
            ",", "6", "]", ";", "}",
        ]),
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

fn to_owned_vec(tokens: &[&str]) -> Vec<String> {
    tokens.iter().map(|item| String::from(*item)).collect()
}
