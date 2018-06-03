use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use syntax;
use syntax::ast::{ItemKind, Mod};
use syntax::codemap::{FileName, FilePathMapping, Pos};
use syntax::parse::{lexer::StringReader, parse_crate_from_source_str, token::Token, ParseSess};
use syntax::tokenstream::TokenStream;

use super::{TestStep, TestStepFactory};
use config::{Config, Profile};
use error::{Result, TestingError};
use utils::SourceCodeAnalyser;

pub struct CheckExpansionStepFactory;
pub struct CheckExpansionStep {
    crate_path: PathBuf,
}

struct RawExpectedExpansion {
    pub module: String,
    pub tokens: Tokens,
}

type Tokens = Vec<String>;
type GroupedExpectedExpansion = Vec<Tokens>;
type ExpectedExpansions = BTreeMap<String, GroupedExpectedExpansion>;
type ActualExpansions = BTreeMap<String, Tokens>;

impl TestStepFactory for CheckExpansionStepFactory {
    fn initialize(&self, _config: &Config, crate_path: &Path) -> Result<Box<TestStep>> {
        Ok(Box::new(CheckExpansionStep::new(
            crate_path.into(),
            Self::find_expected_expansion(crate_path)?,
        )))
    }
}

impl TestStep for CheckExpansionStep {
    fn execute(&self, config: &Config, build_path: &Path) -> Result<()> {
        Self::find_actual_expansion(config, &self.crate_path, build_path)?;

        Ok(())
    }
}

impl CheckExpansionStepFactory {
    pub fn new() -> Self {
        CheckExpansionStepFactory {}
    }

    pub fn find_expected_expansion(crate_path: &Path) -> Result<ExpectedExpansions> {
        let mut result: ExpectedExpansions = BTreeMap::new();

        for item in Self::analyse_crate(crate_path)? {
            if result.contains_key(&item.module) {
                result.get_mut(&item.module).unwrap().push(item.tokens);
            } else {
                result.insert(item.module, vec![item.tokens]);
            }
        }

        Ok(result)
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

    fn parse_tokens(source: String) -> Vec<String> {
        let session = ParseSess::new(FilePathMapping::empty());

        let mut tokens = Vec::new();
        let mut reader = StringReader::new(
            &session,
            session.codemap().new_filemap(FileName::Anon, source),
            None,
        );

        loop {
            match reader.try_next_token() {
                Ok(token) => match token.tok {
                    Token::Eof => {
                        break;
                    }

                    Token::Whitespace => {
                        continue;
                    }

                    _ => {
                        tokens.push(TokenStream::from(token.tok).to_string());
                    }
                },

                Err(_) => {
                    // TODO: when can this happend?
                    break;
                }
            }
        }

        tokens
    }
}

impl SourceCodeAnalyser<RawExpectedExpansion> for CheckExpansionStepFactory {
    fn analyse_source_line(
        _: &[RawExpectedExpansion],
        path: &Path,
        line: (usize, &str),
    ) -> Result<Option<RawExpectedExpansion>> {
        lazy_static! {
            static ref EXPANSION_REGEX: Regex = Regex::new(r"// *~ +EXPAND +(.+)").unwrap();
        }

        if let Some(captures) = EXPANSION_REGEX.captures(line.1) {
            Ok(Some(RawExpectedExpansion {
                module: Self::transform_path_into_module(path),
                tokens: syntax::with_globals(|| {
                    CheckExpansionStepFactory::parse_tokens(captures[1].into())
                }),
            }))
        } else {
            Ok(None)
        }
    }
}

impl CheckExpansionStep {
    pub fn new(crate_path: PathBuf, _expected_expansions: ExpectedExpansions) -> Self {
        CheckExpansionStep { crate_path }
    }

    pub fn find_actual_expansion(
        config: &Config,
        crate_path: &Path,
        build_path: &Path,
    ) -> Result<ActualExpansions> {
        let mut command = Command::new(&config.cargo_command);

        command.current_dir(&crate_path);
        command.env("CARGO_TARGET_DIR", build_path);
        command.arg("rustc");

        if let Some(target) = config.target.as_ref() {
            command.args(&["--target", target]);
        }

        if config.profile == Profile::Release {
            command.arg("--release");
        }

        command.args(&["--", "-Zunpretty=hir"]);

        for (key, value) in &config.cargo_env {
            command.env(key, value);
        }

        let raw_output = command.output()?;
        let stderr = String::from_utf8_lossy(&raw_output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&raw_output.stdout).into_owned();

        match raw_output.status.success() {
            false => bail!(TestingError::CrateBuildFailed { stdout, stderr }),
            true => syntax::with_globals(|| {
                Self::analyse_actual_expansion(stdout, &crate_path.join("src"))
            }),
        }
    }

    fn analyse_actual_expansion(code: String, crate_path: &Path) -> Result<ActualExpansions> {
        let session = ParseSess::new(FilePathMapping::empty());

        let ast = {
            parse_crate_from_source_str(FileName::Anon, code.clone(), &session)
                .map_err(|_| TestingError::UnableToParseExpansion)?
        };

        let expansions = Self::analyse_module_expansion_recursive(&ast.module, crate_path, code, 0)
            .into_iter()
            .map(|item| (Self::normalize_module_name(item.0), item.1))
            .collect();

        Ok(expansions)
    }

    fn analyse_module_expansion_recursive(
        module: &Mod,
        path: &Path,
        mut code: String,
        mut offset: usize,
    ) -> ActualExpansions {
        let mut result = BTreeMap::new();

        for item in &module.items {
            if let ItemKind::Mod(ref item_mod) = item.node {
                let mut next_path = None;

                if next_path.is_none() {
                    if path.join(format!("{}.rs", item.ident)).exists() {
                        next_path = Some(path.into());
                    }
                }

                if next_path.is_none() {
                    if path.join(format!("{}/mod.rs", item.ident)).exists() {
                        next_path = Some(path.join(&item.ident.to_string()));
                    }
                }

                if let Some(next_path) = next_path {
                    let outer_start = item.span.lo().to_usize() - offset;
                    let inner_start = item_mod.inner.lo().to_usize() - offset;

                    let outer_len = item.span.hi().to_usize() - item.span.lo().to_usize();
                    let inner_len =
                        item_mod.inner.hi().to_usize() - item_mod.inner.lo().to_usize() - 1; // TODO: investigate about weird "len - 1" thing

                    let mut nested_expansions = Self::analyse_module_expansion_recursive(
                        item_mod,
                        &next_path,
                        String::from(&code[inner_start..inner_start + inner_len]),
                        item_mod.inner.lo().to_usize(),
                    );

                    result.append(
                        &mut nested_expansions
                            .into_iter()
                            .map(|(key, value)| (item.ident.to_string() + "/" + &key, value))
                            .collect(),
                    );

                    offset += outer_len;
                    code = String::from(&code[0..outer_start]) + &code[outer_start + outer_len..];
                }
            }
        }

        result.insert(
            "mod".into(),
            CheckExpansionStepFactory::parse_tokens(code.clone()),
        );

        result
    }

    fn normalize_module_name(name: String) -> String {
        if name == "mod" {
            "lib".into()
        } else if name.ends_with("/mod") {
            name.chars().take(name.len() - 4).collect()
        } else if name.ends_with("mod") {
            name.chars().take(name.len() - 3).collect()
        } else {
            name
        }
    }
}
