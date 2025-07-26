use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::{externs::ExternMap, property_names::LocalPropertyMap};
pub use globals::GlobalCategory;
pub use property_names::PropertyMap;

pub mod annotation;
pub(crate) mod chunk;
pub(crate) mod context;
pub mod externs;
pub(crate) mod globals;
pub(crate) mod module;
pub(crate) mod property_names;
pub(crate) mod statements;

#[derive(Default, Debug)]
pub struct OptimizerOptions {
    pub hoist: bool,
    pub dedupe: bool,
    pub globals: GlobalsOptions,
    pub externs: ExternsOptions,
    pub rename_properties: bool,
}

#[derive(Default, Debug)]
pub struct GlobalsOptions {
    pub include: GlobalCategory,
    pub hoist: bool,
    pub singletons: bool,
}

#[derive(Default, Debug)]
pub struct ExternsOptions {
    pub inline_const_values: bool,
}

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
    property_map: &PropertyMap,
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

    chunk::optimize_chunk(
        &mut program,
        options,
        LocalPropertyMap::new(property_map),
        &allocator,
        scoping,
    );

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
