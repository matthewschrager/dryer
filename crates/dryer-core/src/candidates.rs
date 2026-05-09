use std::fs;
use std::path::PathBuf;

use rayon::prelude::*;

use crate::config::Config;
use crate::files::SourceFile;
use crate::fingerprints::Fingerprints;
use crate::language::Language;
use crate::normalize::normalize_node;
use crate::parse::parse_source;
use crate::{DryerError, Result};

#[derive(Clone, Debug)]
pub struct Candidate {
    pub file: PathBuf,
    pub language: Language,
    pub kind: String,
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub nodes: usize,
    pub fingerprints: Fingerprints,
}

pub fn scan_candidates(config: &Config, files: &[SourceFile]) -> Result<Vec<Candidate>> {
    let scanned = files
        .par_iter()
        .map(|file| scan_file(config, file))
        .collect::<Result<Vec<_>>>()?;

    Ok(scanned.into_iter().flatten().collect())
}

fn scan_file(config: &Config, file: &SourceFile) -> Result<Vec<Candidate>> {
    let source = fs::read_to_string(&file.path).map_err(|source| DryerError::ReadFile {
        path: file.path.display().to_string(),
        source,
    })?;
    let tree = parse_source(file.language, &source).map_err(|err| match err {
        DryerError::Parse { .. } => DryerError::Parse {
            path: file.path.display().to_string(),
        },
        other => other,
    })?;

    let mut candidates = Vec::new();
    collect_candidates(tree.root_node(), &source, file, config, &mut candidates);
    Ok(candidates)
}

fn collect_candidates(
    node: tree_sitter::Node<'_>,
    source: &str,
    file: &SourceFile,
    config: &Config,
    candidates: &mut Vec<Candidate>,
) {
    if is_candidate_node(file.language, node) && has_body(node) {
        if let Some(candidate) = build_candidate(node, source, file, config) {
            candidates.push(candidate);
        }
    }

    for index in 0..node.named_child_count() {
        let Some(child) = node.named_child(index) else {
            continue;
        };
        collect_candidates(child, source, file, config, candidates);
    }
}

fn build_candidate(
    node: tree_sitter::Node<'_>,
    source: &str,
    file: &SourceFile,
    config: &Config,
) -> Option<Candidate> {
    let normalized = normalize_node(node, source, &config.normalization)?;
    let nodes = normalized.node_count();
    let start_line = node.start_position().row + 1;
    let end_line = node.end_position().row + 1;
    let lines = end_line.saturating_sub(start_line) + 1;

    if lines < config.min_lines || nodes < config.min_nodes {
        return None;
    }

    Some(Candidate {
        file: file.path.clone(),
        language: file.language,
        kind: candidate_kind(file.language, node).to_string(),
        name: candidate_name(node, source),
        start_line,
        end_line,
        nodes,
        fingerprints: Fingerprints::from_normalized(&normalized),
    })
}

fn is_candidate_node(language: Language, node: tree_sitter::Node<'_>) -> bool {
    match language {
        Language::Rust => matches!(node.kind(), "function_item" | "closure_expression"),
        Language::TypeScript | Language::Tsx => matches!(
            node.kind(),
            "function_declaration" | "method_definition" | "arrow_function"
        ),
        Language::Haskell | Language::Daml => matches!(
            node.kind(),
            "function" | "bind" | "data_type" | "newtype" | "class" | "instance"
        ),
    }
}

fn has_body(node: tree_sitter::Node<'_>) -> bool {
    match node.kind() {
        "function_item" => child_kind_exists(node, "block"),
        "function_declaration" | "method_definition" => child_kind_exists(node, "statement_block"),
        "arrow_function" | "closure_expression" => true,
        "function" | "bind" => {
            child_kind_exists(node, "match") || node.child_by_field_name("expression").is_some()
        }
        "data_type" | "newtype" => {
            child_kind_exists(node, "data_constructors")
                || child_kind_exists(node, "gadt_constructors")
        }
        "class" => child_kind_exists(node, "class_declarations"),
        "instance" => child_kind_exists(node, "instance_declarations"),
        _ => true,
    }
}

fn child_kind_exists(node: tree_sitter::Node<'_>, kind: &str) -> bool {
    for index in 0..node.named_child_count() {
        if node
            .named_child(index)
            .is_some_and(|child| child.kind() == kind)
        {
            return true;
        }
    }
    false
}

fn candidate_kind(language: Language, node: tree_sitter::Node<'_>) -> &'static str {
    match (language, node.kind()) {
        (Language::Rust, "function_item") => "function",
        (Language::Rust, "closure_expression") => "closure",
        (Language::Haskell | Language::Daml, "function" | "bind") => "function",
        (Language::Haskell | Language::Daml, "data_type") => "data",
        (Language::Haskell | Language::Daml, "newtype") => "newtype",
        (Language::Haskell | Language::Daml, "class") => "class",
        (Language::Haskell | Language::Daml, "instance") => "instance",
        (_, "function_declaration") => "function",
        (_, "method_definition") => "method",
        (_, "arrow_function") => "arrow_function",
        _ => "code",
    }
}

fn candidate_name(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    if let Some(name) = node.child_by_field_name("name") {
        return text(name, source);
    }

    if node.kind() == "arrow_function" {
        return arrow_name(node, source);
    }

    None
}

fn arrow_name(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    let mut parent = node.parent();
    while let Some(current) = parent {
        if matches!(
            current.kind(),
            "variable_declarator" | "pair" | "assignment_expression"
        ) {
            if let Some(name) = current.child_by_field_name("name") {
                return text(name, source);
            }
            if let Some(key) = current.child_by_field_name("key") {
                return text(key, source);
            }
            if let Some(left) = current.child_by_field_name("left") {
                return text(left, source);
            }
        }
        parent = current.parent();
    }
    None
}

fn text(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    node.utf8_text(source.as_bytes()).ok().map(str::to_string)
}
