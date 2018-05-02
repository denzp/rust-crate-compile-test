#![deny(warnings)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate tempfile;
extern crate walkdir;

pub mod cargo_messages;
pub mod config;
pub mod error;
pub mod steps;

mod plan;

pub mod prelude {
    pub use config::{Config, Mode};
    pub use error::{Result, TestingError};
    pub use plan::run_tests;
}
