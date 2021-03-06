use regex::Regex;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use crate_compile_test::config::{Config, Mode};
use crate_compile_test::steps::TestStepFactory;

use crate_compile_test::steps::check_errors::{
    CheckErrorsStepFactory, CompilerMessage, DiagnosticLevel, MessageLocation, MessageType,
};

#[test]
fn it_should_handle_success() {
    let step = CheckErrorsStepFactory::new();
    let output_path = tempdir().unwrap();

    let config = Config::new(Mode::BuildSuccess, "example/tests/build-success");

    // TODO: it should throw ealier - during initialization - because there are no expected messages

    let error = {
        step.initialize(&config, &Path::new("example/tests/build-success/success-1"))
            .unwrap()
            .execute(&config, output_path.as_ref())
            .expect_err("It should throw error")
    };

    assert_eq!(error.to_string(), "Unexpectedly successful build!");
}

#[test]
fn it_should_handle_fail() {
    let step = CheckErrorsStepFactory::new();
    let output_path = tempdir().unwrap();

    let config = Config::new(Mode::BuildSuccess, "example/tests/build-fail");

    let error = {
        step.initialize(&config, &Path::new("example/tests/build-fail/fail-1"))
            .unwrap()
            .execute(&config, output_path.as_ref())
            .expect_err("It should fail building the crate")
    };

    assert_eq!(
        error.to_string(),
        read_output!("tests/ui/fail-1.check_errors.output")
    );
}

#[test]
fn it_should_handle_all_matched_messages() {
    let step = CheckErrorsStepFactory::new();
    let output_path = tempdir().unwrap();

    let config = Config::new(Mode::BuildSuccess, "example/tests/build-fail");

    step.initialize(&config, &Path::new("example/tests/build-fail/fail-2"))
        .unwrap()
        .execute(&config, output_path.as_ref())
        .expect("It should finish without error");
}

#[test]
fn it_should_collect_expected_messages() {
    let crate_path = Path::new("example/tests/build-fail/fail-1");
    let messages = CheckErrorsStepFactory::collect_crate_messages(&crate_path).unwrap();

    assert_eq!(
        messages,
        &[
            CompilerMessage {
                message: MessageType::Text("another warning".into()),
                code: None,

                level: DiagnosticLevel::Warning,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 2
                })
            },
            CompilerMessage {
                message: MessageType::None,
                code: Some("E0432".into()),

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 2
                })
            },
            CompilerMessage {
                message: MessageType::Text("unresolved import `mod2::func3`".into()),
                code: None,

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 2
                })
            },
            CompilerMessage {
                message: MessageType::None,
                code: Some("E0433".into()),

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 12
                })
            },
            CompilerMessage {
                message: MessageType::Text("With extra space".into()),
                code: None,

                level: DiagnosticLevel::Note,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 17
                })
            },
            CompilerMessage {
                message: MessageType::Text("For previous line".into()),
                code: None,

                level: DiagnosticLevel::Help,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 17
                })
            },
        ]
    );
}

#[test]
fn it_should_handle_nested_sources() {
    let step = CheckErrorsStepFactory::new();
    let output_path = tempdir().unwrap();

    let config = Config::new(Mode::BuildSuccess, "example/tests/build-fail");

    let error = {
        step.initialize(&config, &Path::new("example/tests/build-fail/fail-3"))
            .unwrap()
            .execute(&config, output_path.as_ref())
            .expect_err("It should fail building the crate")
    };

    assert_eq!(
        error.to_string(),
        read_output!("tests/ui/fail-3.check_errors.output")
    );
}

#[test]
fn it_should_collect_messages_from_nested_sources() {
    let crate_path = Path::new("example/tests/build-fail/fail-3");
    let messages = CheckErrorsStepFactory::collect_crate_messages(&crate_path).unwrap();

    assert_eq!(
        messages,
        &[
            CompilerMessage {
                message: MessageType::None,
                code: Some("E0308".into()),

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 4
                })
            },
            CompilerMessage {
                message: MessageType::Text("function `func1` is private".into()),
                code: None,

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/lib.rs"),
                    line: 6
                })
            },
            CompilerMessage {
                message: MessageType::None,
                code: Some("E0433".into()),

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/mod_2/mod.rs"),
                    line: 1
                })
            },
            CompilerMessage {
                message: MessageType::Text("With extra space".into()),
                code: None,

                level: DiagnosticLevel::Note,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/mod_2/mod.rs"),
                    line: 6
                })
            },
            CompilerMessage {
                message: MessageType::Text("For previous line".into()),
                code: None,

                level: DiagnosticLevel::Help,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/mod_2/mod.rs"),
                    line: 6
                })
            },
            CompilerMessage {
                message: MessageType::Text("another warning".into()),
                code: None,

                level: DiagnosticLevel::Warning,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/mod_1.rs"),
                    line: 1
                })
            },
            CompilerMessage {
                message: MessageType::None,
                code: Some("E0432".into()),

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/mod_1.rs"),
                    line: 1
                })
            },
            CompilerMessage {
                message: MessageType::Text("unresolved import `mod_2::func3`".into()),
                code: None,

                level: DiagnosticLevel::Error,
                location: Some(MessageLocation {
                    file: PathBuf::from("src/mod_1.rs"),
                    line: 1
                })
            },
        ]
    );
}

#[test]
fn it_should_use_cargo_from_config() {
    let step = CheckErrorsStepFactory::new();

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
    let step = CheckErrorsStepFactory::new();
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
    let step = CheckErrorsStepFactory::new();
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

#[test]
fn it_should_collect_global_messages() {
    let crate_path = Path::new("example/tests/build-fail/fail-4");
    let messages = CheckErrorsStepFactory::collect_crate_messages(&crate_path).unwrap();

    assert_eq!(
        messages,
        &[
            CompilerMessage {
                message: MessageType::Regex(Regex::new("-l(.+)libcore-(.+)\\.rlib").unwrap()),
                code: None,

                level: DiagnosticLevel::Note,
                location: None,
            },
            CompilerMessage {
                message: MessageType::Regex(
                    Regex::new(r#"undefined reference to `some_external_fn'"#).unwrap()
                ),
                code: None,

                level: DiagnosticLevel::Note,
                location: None,
            },
            CompilerMessage {
                message: MessageType::Regex(
                    Regex::new(r#"undefined reference to `third_external_fn'"#).unwrap()
                ),
                code: None,

                level: DiagnosticLevel::Note,
                location: None,
            },
        ]
    );
}

#[test]
fn it_should_global_errors() {
    let step = CheckErrorsStepFactory::new();
    let output_path = tempdir().unwrap();

    let config = Config::new(Mode::BuildSuccess, "example/tests/build-fail");

    let error = {
        step.initialize(&config, &Path::new("example/tests/build-fail/fail-4"))
            .unwrap()
            .execute(&config, output_path.as_ref())
            .expect_err("It should fail building the crate")
    };

    assert_eq!(
        error.to_string(),
        read_output!("tests/ui/fail-4.check_errors.output")
    );
}
