extern crate crate_compile_test;

use crate_compile_test::prelude::*;

#[test]
fn run_build_fail_tests() {
    run_tests(Config::new(Mode::BuildFail, "example-tests/build-fail"));
}

#[test]
fn run_build_success_tests() {
    run_tests(Config::new(
        Mode::BuildSuccess,
        "example-tests/build-success",
    ));
}

#[test]
fn run_expand_tests() {
    run_tests(Config::new(Mode::Expand, "example-tests/expand"));
}
