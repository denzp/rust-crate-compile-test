use std::path::Path;

use regex::Regex;

use error::Result;
use utils::SourceCodeAnalyser;

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

impl SourceCodeAnalyser<ExpectedExpansion> for CheckExpansionStepFactory {
    fn analyse_source_line(
        _previous: &[ExpectedExpansion],
        path: &Path,
        line: (usize, &str),
    ) -> Result<Option<ExpectedExpansion>> {
        lazy_static! {
            static ref EXPANSION_REGEX: Regex = Regex::new(r"// *~ +EXPAND +(.+)").unwrap();
        }

        if let Some(captures) = EXPANSION_REGEX.captures(line.1) {
            Ok(Some(ExpectedExpansion {
                module: Self::transform_path_into_module(path),
                expansion: captures[1].into(),
            }))
        } else {
            Ok(None)
        }
    }
}
