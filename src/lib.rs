#![deny(warnings)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tempfile;
extern crate walkdir;

pub mod config;
pub mod error;
pub mod steps;

mod cargo_messages;
mod formatting;
mod plan;

pub mod prelude {
    pub use config::{Config, Mode, Profile};
    pub use error::{Result, TestingError};
    pub use plan::{run_tests, run_tests_with_writer};
}
