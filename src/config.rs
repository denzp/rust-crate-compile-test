use std::path::{Path, PathBuf};

pub enum Mode {
    BuildFail,
    BuildSuccess,
    Expand,
}

pub struct Config {
    pub mode: Mode,

    pub base_dir: PathBuf,
    pub target: Option<String>,

    pub cargo_env: Vec<(String, String)>,
    pub cargo_command: String,
}

impl Config {
    pub fn new<P: AsRef<Path>>(mode: Mode, base_dir: P) -> Self {
        Config {
            mode,

            base_dir: base_dir.as_ref().into(),
            target: None,

            cargo_env: vec![],
            cargo_command: "cargo".into(),
        }
    }

    pub fn add_cargo_env<S: Into<String>>(&mut self, key: S, value: S) {
        self.cargo_env.push((key.into(), value.into()));
    }
}
