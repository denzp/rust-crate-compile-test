use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use failure::ResultExt;
use regex::Regex;
use walkdir::WalkDir;

use error::Result;

#[derive(Debug, PartialEq)]
pub struct ExpectedExpansion {
    pub module: String,
    pub expansion: String,
}

pub struct CheckExpansionStepFactory;

impl CheckExpansionStepFactory {
    pub fn new() -> Self {
        CheckExpansionStepFactory {}
    }

    pub fn collect_expectations(crate_path: &Path) -> Result<Vec<ExpectedExpansion>> {
        let sources = WalkDir::new(&crate_path.join("src"))
            .into_iter()
            .map(|entry| entry.unwrap())
            .filter_map(
                |entry| match entry.path().extension().and_then(|item| item.to_str()) {
                    Some("rs") => Some(PathBuf::from(entry.path())),
                    _ => None,
                },
            );

        let mut messages = vec![];

        for path in sources {
            let source_path = path.strip_prefix(crate_path)?;
            let source_file = BufReader::new({
                File::open(&path).context(format!("Unable to open source at {:?}", path))?
            });

            messages.append({
                &mut source_file
                    .lines()
                    .filter_map(|line| Self::analyse_source_line(&source_path, &line.unwrap()))
                    .collect()
            });
        }

        Ok(messages)
    }

    fn analyse_source_line(path: &Path, line: &str) -> Option<ExpectedExpansion> {
        lazy_static! {
            static ref EXPANSION_REGEX: Regex = Regex::new(r"// *~ +EXPAND +(.+)").unwrap();
        }

        if let Some(captures) = EXPANSION_REGEX.captures(line) {
            Some(ExpectedExpansion {
                module: Self::transform_path_into_module(path),
                expansion: captures[1].into(),
            })
        } else {
            None
        }
    }

    fn transform_path_into_module(path: &Path) -> String {
        lazy_static! {
            static ref MODULE_NAME_REGEX: Regex = Regex::new(r"src/(.+?)(/mod)?.rs").unwrap();
        }

        match MODULE_NAME_REGEX.captures(&path.to_string_lossy()) {
            Some(captures) => captures[1].into(),
            None => path.to_string_lossy().into(),
        }
    }
}
