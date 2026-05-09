use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{DryerError, Result};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageFilter {
    #[default]
    All,
    Rust,
    TypeScript,
    Haskell,
    Daml,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NameMode {
    Loose,
    #[default]
    Balanced,
    Strict,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizationConfig {
    pub name_mode: NameMode,
    pub preserve_call_names: bool,
    pub preserve_property_names: bool,
    pub preserve_type_names: bool,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            name_mode: NameMode::Balanced,
            preserve_call_names: false,
            preserve_property_names: false,
            preserve_type_names: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub paths: Vec<PathBuf>,
    pub threshold: f64,
    pub min_lines: usize,
    pub min_nodes: usize,
    pub language: LanguageFilter,
    pub format: OutputFormat,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub cross_language: bool,
    pub max_candidates: Option<usize>,
    pub normalization: NormalizationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paths: vec![PathBuf::from(".")],
            threshold: 0.82,
            min_lines: 6,
            min_nodes: 35,
            language: LanguageFilter::All,
            format: OutputFormat::Text,
            include: Vec::new(),
            exclude: default_excludes(),
            cross_language: false,
            max_candidates: None,
            normalization: NormalizationConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConfigFile {
    pub threshold: Option<f64>,
    pub min_lines: Option<usize>,
    pub min_nodes: Option<usize>,
    pub language: Option<LanguageFilter>,
    pub format: Option<OutputFormat>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub cross_language: Option<bool>,
    pub max_candidates: Option<usize>,
    pub normalization: Option<NormalizationConfig>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| DryerError::ReadConfig {
            path: path.display().to_string(),
            source,
        })?;
        let file =
            toml::from_str::<ConfigFile>(&text).map_err(|source| DryerError::ParseConfig {
                path: path.display().to_string(),
                source,
            })?;
        Ok(Self::default().merge_file(file))
    }

    pub fn merge_file(mut self, file: ConfigFile) -> Self {
        if let Some(threshold) = file.threshold {
            self.threshold = threshold;
        }
        if let Some(min_lines) = file.min_lines {
            self.min_lines = min_lines;
        }
        if let Some(min_nodes) = file.min_nodes {
            self.min_nodes = min_nodes;
        }
        if let Some(language) = file.language {
            self.language = language;
        }
        if let Some(format) = file.format {
            self.format = format;
        }
        if let Some(include) = file.include {
            self.include = include;
        }
        if let Some(exclude) = file.exclude {
            self.exclude = exclude;
        }
        if let Some(cross_language) = file.cross_language {
            self.cross_language = cross_language;
        }
        if let Some(max_candidates) = file.max_candidates {
            self.max_candidates = Some(max_candidates);
        }
        if let Some(normalization) = file.normalization {
            self.normalization = normalization;
        }
        self
    }
}

pub fn default_excludes() -> Vec<String> {
    [
        "**/.git/**",
        "**/target/**",
        "**/node_modules/**",
        "**/dist/**",
        "**/build/**",
        "**/.next/**",
        "**/coverage/**",
        "**/*.generated.ts",
        "**/*.generated.tsx",
        "**/*.gen.ts",
        "**/*.gen.tsx",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
