#![deny(warnings)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate colored;
extern crate regex;
extern crate rustfmt_nightly as rustfmt;
extern crate serde;
extern crate serde_json;
extern crate syntax;
extern crate tempfile;
extern crate walkdir;

pub mod config;
pub mod error;
pub mod steps;
pub mod utils;

mod cargo_messages;
mod formatting;
mod plan;
mod runner;

pub mod prelude {
    pub use config::{Config, Mode, Profile};
    pub use error::{Result, TestingError};
    pub use runner::TestRunner;
}

#[macro_export]
macro_rules! bootstrap_compilation_tests {
    ($($name:ident),+) => {
        fn main() {
            use std::process::exit;
            use std::io::stdout;

            let mut output = stdout();
            let mut runner = TestRunner::new(&mut output);

            $($name(&mut runner);)+

            if !runner.start().unwrap().is_success() {
                exit(1);
            }
        }
    };
}
