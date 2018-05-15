#[macro_use]
extern crate crate_compile_test;

use crate_compile_test::prelude::*;

crate_compile_test_suite! {
    "build-fail tests" => {
        run_compile_tests!(Config::new(Mode::BuildFail, "tests/build-fail"));
    },

    "build-success tests" => {
        run_compile_tests!(Config::new(Mode::BuildSuccess, "tests/build-success"));
    },

    "expansion tests" => {
        run_compile_tests!(Config::new(Mode::Expand, "tests/expand"));
    }
}
