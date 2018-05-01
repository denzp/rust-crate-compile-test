#![deny(warnings)]

#[macro_use]
extern crate failure;
extern crate tempfile;
extern crate walkdir;

pub mod config;
pub mod error;
pub mod steps;

mod plan;

pub mod prelude {
    pub use config::{Config, Mode};
    pub use error::{Result, TestingError};
    pub use plan::run_tests;
}
