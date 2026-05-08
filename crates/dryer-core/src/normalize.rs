use crate::config::{NameMode, NormalizationConfig};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Normalized {
    label: String,
    children: Vec<Normalized>,
}

impl Normalized {
    pub fn new(label: impl Into<String>, children: Vec<Normalized>) -> Self {
        Self {
            label: label.into(),
            children,
        }
    }

    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(Self::node_count).sum::<usize>()
    }

    pub fn repr(&self) -> String {
        if self.children.is_empty() {
            return self.label.clone();
        }

        let mut repr = String::new();
        repr.push('(');
        repr.push_str(&self.label);
        for child in &self.children {
            repr.push(' ');
            repr.push_str(&child.repr());
        }
        repr.push(')');
        repr
    }

    pub fn child_reprs(&self, out: &mut Vec<String>) {
        out.push(self.repr());
        for child in &self.children {
            child.child_reprs(out);
        }
    }
}

pub fn normalize_node(
    node: tree_sitter::Node<'_>,
    source: &str,
    config: &NormalizationConfig,
) -> Option<Normalized> {
    if !node.is_named() {
        return meaningful_token(node.kind())
            .map(|label| Normalized::new(format!("token:{label}"), Vec::new()));
    }

    if node.kind() == "comment" {
        return None;
    }

    let mut children = Vec::new();
    for index in 0..node.child_count() {
        let Some(child) = node.child(index) else {
            continue;
        };
        if let Some(normalized) = normalize_node(child, source, config) {
            children.push(normalized);
        }
    }

    Some(Normalized::new(label_for(node, source, config), children))
}

fn label_for(node: tree_sitter::Node<'_>, source: &str, config: &NormalizationConfig) -> String {
    let kind = node.kind();
    if kind == "macro_invocation" {
        if let Some(name) = first_identifier_text(node, source) {
            return format!("macro:{name}");
        }
    }

    if is_identifier(kind) {
        return match config.name_mode {
            NameMode::Strict => format!("ident:{}", node_text(node, source)),
            NameMode::Loose | NameMode::Balanced => "ident".to_string(),
        };
    }

    if is_literal(kind) {
        return literal_label(kind).to_string();
    }

    kind.to_string()
}

fn node_text(node: tree_sitter::Node<'_>, source: &str) -> String {
    node.utf8_text(source.as_bytes())
        .unwrap_or_default()
        .to_string()
}

fn first_identifier_text(node: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    for index in 0..node.named_child_count() {
        let Some(child) = node.named_child(index) else {
            continue;
        };
        if is_identifier(child.kind()) {
            return Some(node_text(child, source));
        }
    }
    None
}

fn is_identifier(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "field_identifier"
            | "property_identifier"
            | "shorthand_property_identifier"
            | "type_identifier"
            | "scoped_type_identifier"
            | "namespace_identifier"
            | "label_identifier"
    )
}

fn is_literal(kind: &str) -> bool {
    kind.contains("literal")
        || matches!(
            kind,
            "string"
                | "template_string"
                | "number"
                | "regex"
                | "true"
                | "false"
                | "null"
                | "undefined"
        )
}

fn literal_label(kind: &str) -> &'static str {
    if kind.contains("string") || kind == "template_string" {
        "literal:string"
    } else if kind.contains("char") {
        "literal:char"
    } else if kind.contains("number") || kind.contains("integer") || kind.contains("float") {
        "literal:number"
    } else if matches!(kind, "true" | "false") {
        "literal:boolean"
    } else if matches!(kind, "null" | "undefined") {
        "literal:nullish"
    } else {
        "literal"
    }
}

fn meaningful_token(kind: &str) -> Option<&'static str> {
    match kind {
        "+" => Some("+"),
        "-" => Some("-"),
        "*" => Some("*"),
        "/" => Some("/"),
        "%" => Some("%"),
        "==" | "===" => Some("eq"),
        "!=" | "!==" => Some("neq"),
        "<" => Some("<"),
        ">" => Some(">"),
        "<=" => Some("<="),
        ">=" => Some(">="),
        "&&" => Some("&&"),
        "||" => Some("||"),
        "!" => Some("!"),
        "??" => Some("??"),
        "?" => Some("?"),
        "=" => Some("="),
        "+=" => Some("+="),
        "-=" => Some("-="),
        "*=" => Some("*="),
        "/=" => Some("/="),
        "%=" => Some("%="),
        "=>" => Some("=>"),
        ".." => Some(".."),
        "..=" => Some("..="),
        "&" => Some("&"),
        "|" => Some("|"),
        "^" => Some("^"),
        "<<" => Some("<<"),
        ">>" => Some(">>"),
        _ => None,
    }
}
