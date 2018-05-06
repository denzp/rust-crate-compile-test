#[macro_use]
extern crate pretty_assertions;

#[macro_use]
mod utils;

extern crate crate_compile_test;
use crate_compile_test::prelude::*;

#[test]
fn it_should_report_failure() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();

    run_tests_with_writer(
        Config::new(Mode::BuildFail, "example/tests/build-fail"),
        &mut actual_output_bytes,
    ).unwrap_err();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.failure.output")
    );
}

#[test]
fn it_should_report_about_unexpected_success() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();

    run_tests_with_writer(
        Config::new(Mode::BuildFail, "example/tests/build-success"),
        &mut actual_output_bytes,
    ).unwrap_err();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.unexpected_success.output")
    );
}

#[test]
fn it_should_report_success() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();

    run_tests_with_writer(
        Config::new(Mode::BuildSuccess, "example/tests/build-success"),
        &mut actual_output_bytes,
    ).unwrap();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.success.output")
    );
}

#[test]
fn it_should_report_unexpected_failure() {
    let mut actual_output_bytes: Vec<u8> = Vec::new();

    run_tests_with_writer(
        Config::new(Mode::BuildSuccess, "example/tests/build-fail"),
        &mut actual_output_bytes,
    ).unwrap_err();

    assert_eq!(
        String::from_utf8_lossy(&actual_output_bytes),
        read_output!("tests/ui/complete.unexpected_failure.output")
    );
}
