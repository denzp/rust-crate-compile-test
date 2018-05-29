use std::path::Path;

use crate_compile_test::steps::check_expansion::{CheckExpansionStepFactory, ExpectedExpansion};
use crate_compile_test::utils::SourceCodeAnalyser;

#[test]
fn it_should_collect_expected_expansions() {
    let crate_path = Path::new("example/tests/expand/expand-1");
    let messages = CheckExpansionStepFactory::analyse_crate(&crate_path).unwrap();

    assert_eq!(
        messages,
        &[
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "x = 2;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "x = 1 + 1;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "y = 1 + 1;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "y = 2 + 2;".into(),
            },
            ExpectedExpansion {
                module: "lib".into(),
                expansion: "y = 4 + 4;".into(),
            },
            ExpectedExpansion {
                module: "mod_2".into(),
                expansion: "pub fn other_fn(_arg: u32) -> u32 {".into(),
            },
            ExpectedExpansion {
                module: "mod_2".into(),
                expansion: "0".into(),
            },
            ExpectedExpansion {
                module: "mod_2".into(),
                expansion: "}".into(),
            },
            ExpectedExpansion {
                module: "mod_2/nested_mod_1".into(),
                expansion: "t = [4, 3, 2, 6];".into(),
            },
            ExpectedExpansion {
                module: "mod_1".into(),
                expansion: "pub fn div_n(a: f64) -> f64 {".into(),
            },
            ExpectedExpansion {
                module: "mod_1".into(),
                expansion: "a / 4 as f64".into(),
            },
            ExpectedExpansion {
                module: "mod_1".into(),
                expansion: "}".into(),
            },
        ]
    );
}
