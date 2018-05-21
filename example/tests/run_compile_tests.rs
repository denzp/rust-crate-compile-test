#[macro_use]
extern crate crate_compile_test;

use crate_compile_test::prelude::*;

fn fail_tests(tester: &mut TestRunner) {
    tester.add("build-fail tests", || {
        Config::new(Mode::BuildFail, "tests/build-fail")
    });
}

fn success_tests(tester: &mut TestRunner) {
    tester.add("build-success tests", || {
        Config::new(Mode::BuildSuccess, "tests/build-success")
    });
}

fn expansion_tests(tester: &mut TestRunner) {
    tester.add("expansion tests", || {
        Config::new(Mode::Expand, "tests/expand")
    });
}

bootstrap_compilation_tests![fail_tests, success_tests, expansion_tests];
