use std::path::{Path, PathBuf};

use steps::TestStepFactory;

#[derive(PartialEq)]
pub enum Mode {
    BuildFail,
    BuildSuccess,
    Expand,
}

#[derive(PartialEq)]
pub enum Profile {
    Debug,
    Release,
}

pub struct Config {
    pub mode: Mode,

    pub base_dir: PathBuf,
    pub target: Option<String>,
    pub profile: Profile,

    pub cargo_env: Vec<(String, String)>,
    pub cargo_command: String,

    pub crates_filter: Box<Fn(&Path) -> bool>,
    pub additional_steps: Vec<Box<TestStepFactory>>,
}

impl Config {
    pub fn new<P: AsRef<Path>>(mode: Mode, base_dir: P) -> Self {
        Config {
            mode,

            base_dir: base_dir.as_ref().into(),
            target: None,
            profile: Profile::Release,

            cargo_env: vec![],
            cargo_command: "cargo".into(),

            crates_filter: Box::new(|_| true),
            additional_steps: vec![],
        }
    }

    pub fn add_cargo_env<S: Into<String>>(&mut self, key: S, value: S) {
        self.cargo_env.push((key.into(), value.into()));
    }
}
