use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
}

impl Language {
    pub fn detect(path: &Path) -> Option<Self> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => Some(Self::Rust),
            Some("ts") => {
                if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".d.ts"))
                {
                    None
                } else {
                    Some(Self::TypeScript)
                }
            }
            Some("tsx") => Some(Self::Tsx),
            _ => None,
        }
    }

    pub fn report_name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
        }
    }

    pub fn family(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript | Self::Tsx => "typescript",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.report_name())
    }
}
