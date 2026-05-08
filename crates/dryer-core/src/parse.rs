use tree_sitter::{Language as TsLanguage, Parser, Tree};

use crate::language::Language;
use crate::{DryerError, Result};

pub fn parse_source(language: Language, source: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    let grammar = grammar_for(language);
    parser
        .set_language(&grammar)
        .map_err(|err| DryerError::ParserLanguage {
            language,
            message: err.to_string(),
        })?;
    parser.parse(source, None).ok_or_else(|| DryerError::Parse {
        path: "<memory>".to_string(),
    })
}

fn grammar_for(language: Language) -> TsLanguage {
    match language {
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
    }
}
