mod candidates;
mod config;
mod files;
mod fingerprints;
mod language;
mod matcher;
mod normalize;
mod output;
mod parse;
mod report;

pub use config::{Config, ConfigFile, LanguageFilter, NameMode, NormalizationConfig, OutputFormat};
pub use language::Language;
pub use matcher::find_duplicates;
pub use output::{format_json, format_sarif, format_text};
pub use report::{DuplicateCandidate, Location, Report};

#[derive(Debug, thiserror::Error)]
pub enum DryerError {
    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read config {path}: {source}")]
    ReadConfig {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config {path}: {source}")]
    ParseConfig {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to compile glob pattern {pattern:?}: {source}")]
    Glob {
        pattern: String,
        #[source]
        source: globset::Error,
    },

    #[error("failed to configure parser for {language}: {message}")]
    ParserLanguage { language: Language, message: String },

    #[error("failed to parse {path}")]
    Parse { path: String },
}

pub type Result<T> = std::result::Result<T, DryerError>;
