use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use rustc_hash::FxHashMap;
use serde::Serialize;

use crate::externs::ExternMap;

pub mod annotation;
pub(crate) mod chunk;
pub(crate) mod context;
pub mod externs;
pub(crate) mod globals;
pub(crate) mod module;
pub(crate) mod property_names;
pub(crate) mod statements;

pub use globals::{Globals, add_default_globals};
pub use property_names::generate_unique_names;

#[derive(Default, Debug)]
pub struct OptimizerOptions {
    pub hoist: bool,
    pub dedupe: bool,
    pub hoist_globals: bool,
    pub inline_extern_values: bool,
    pub singletons: bool,
    pub rename_properties: bool,
}

#[derive(Serialize)]
pub struct OptimizerOutput {
    pub code: String,
    pub map: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OptimizerError {
    #[error("Unable to parse javascript file: {0}")]
    SyntaxError(String),
    #[error("Unable to parse javascript file: {0}")]
    SemanticError(String),
    #[error("Unable to optimize javascript file: {0}")]
    OptimizerError(String),
    #[error("Unable to parse property map: {0}")]
    PropertyMapParseError(String),
}

pub fn optimize_module(
    source_text: &str,
    options: &OptimizerOptions,
    externs: &ExternMap,
) -> Result<OptimizerOutput, OptimizerError> {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    if let Some(err) = ret.errors.first() {
        return Err(OptimizerError::SyntaxError(err.to_string()));
    }

    let mut program = ret.program;

    let ret = SemanticBuilder::new().with_excess_capacity(0.1).build(&program);
    if let Some(err) = ret.errors.first() {
        return Err(OptimizerError::SemanticError(err.to_string()));
    }

    let scoping = ret.semantic.into_scoping();
    module::optimize_module(&mut program, options, externs, &allocator, scoping);

    let result = Codegen::new()
        .with_options(CodegenOptions {
            single_quote: false,
            minify: false,
            comments: CommentOptions::default(),
            source_map_path: Some(PathBuf::new()),
        })
        .build(&program);

    Ok(OptimizerOutput {
        code: result.code,
        map: result.map.map_or_else(String::default, |v| v.to_json_string()),
    })
}

pub fn optimize_chunk(
    source_text: &str,
    options: &OptimizerOptions,
    globals: &Globals,
    property_map: &FxHashMap<String, String>,
) -> Result<OptimizerOutput, OptimizerError> {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    if let Some(err) = ret.errors.first() {
        return Err(OptimizerError::SyntaxError(err.to_string()));
    }

    let mut program = ret.program;

    let ret = SemanticBuilder::new().with_excess_capacity(0.1).build(&program);
    if let Some(err) = ret.errors.first() {
        return Err(OptimizerError::SemanticError(err.to_string()));
    }

    let scoping = ret.semantic.into_scoping();
    chunk::optimize_chunk(&mut program, options, globals, property_map, &allocator, scoping);

    let result = Codegen::new()
        .with_options(CodegenOptions {
            single_quote: false,
            minify: false,
            comments: CommentOptions::default(),
            source_map_path: Some(PathBuf::new()),
        })
        .build(&program);

    Ok(OptimizerOutput {
        code: result.code,
        map: result.map.map_or_else(String::default, |v| v.to_json_string()),
    })
}

pub fn collect_property_names(source_text: &str) -> Result<Vec<String>, OptimizerError> {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    if let Some(err) = ret.errors.first() {
        return Err(OptimizerError::SyntaxError(err.to_string()));
    }

    let mut program = ret.program;

    let ret = SemanticBuilder::new().build(&program);
    if let Some(err) = ret.errors.first() {
        return Err(OptimizerError::SemanticError(err.to_string()));
    }

    let scoping = ret.semantic.into_scoping();
    let names = property_names::collect_property_names(&mut program, &allocator, scoping);

    Ok(names.iter().map(|name| name.to_string()).collect())
}

pub fn deserialize_property_map(data: &[u8]) -> Result<FxHashMap<String, String>, OptimizerError> {
    let mut index = FxHashMap::default();
    for (i, line) in data.split(|c| *c == b'\n').enumerate() {
        let line = line.trim_ascii();
        let Ok(line) = str::from_utf8(line) else {
            return Err(OptimizerError::PropertyMapParseError(format!(
                "invalid utf8 at line '{}'",
                i + 1
            )));
        };
        let mut split = line.split('=');
        let Some(key) = split.next() else {
            return Err(OptimizerError::PropertyMapParseError(format!(
                "invalid key at line '{}'",
                i + 1
            )));
        };
        let Some(value) = split.next() else {
            return Err(OptimizerError::PropertyMapParseError(format!(
                "invalid value at line '{}'",
                i + 1
            )));
        };
        index.insert(key.to_string(), value.to_string());
    }
    Ok(index)
}
