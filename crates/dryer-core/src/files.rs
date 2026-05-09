use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

use crate::config::{Config, LanguageFilter};
use crate::language::Language;
use crate::{DryerError, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub language: Language,
}

pub fn discover_files(config: &Config) -> Result<Vec<SourceFile>> {
    let includes = compile_globs(&config.include)?;
    let excludes = compile_globs(&config.exclude)?;
    let mut files = Vec::new();

    for path in &config.paths {
        if path.is_file() {
            maybe_push_file(path, config, &includes, &excludes, &mut files);
            continue;
        }

        let mut builder = WalkBuilder::new(path);
        builder.standard_filters(true);
        builder.hidden(false);

        for entry in builder.build().filter_map(std::result::Result::ok) {
            let file_type = entry.file_type();
            if file_type.is_some_and(|kind| kind.is_file()) {
                maybe_push_file(entry.path(), config, &includes, &excludes, &mut files);
            }
        }
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    files.dedup_by(|left, right| left.path == right.path);
    Ok(files)
}

fn maybe_push_file(
    path: &Path,
    config: &Config,
    includes: &GlobSet,
    excludes: &GlobSet,
    files: &mut Vec<SourceFile>,
) {
    if !config.include.is_empty() && !includes.is_match(path) {
        return;
    }
    if excludes.is_match(path) {
        return;
    }

    let Some(language) = Language::detect(path) else {
        return;
    };
    if !matches_filter(language, &config.language) {
        return;
    }

    files.push(SourceFile {
        path: path.to_path_buf(),
        language,
    });
}

fn matches_filter(language: Language, filter: &LanguageFilter) -> bool {
    match filter {
        LanguageFilter::All => true,
        LanguageFilter::Rust => language == Language::Rust,
        LanguageFilter::TypeScript => {
            matches!(language, Language::TypeScript | Language::Tsx)
        }
        LanguageFilter::Haskell => language == Language::Haskell,
        LanguageFilter::Daml => language == Language::Daml,
    }
}

fn compile_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).map_err(|source| DryerError::Glob {
            pattern: pattern.clone(),
            source,
        })?;
        builder.add(glob);
    }
    builder.build().map_err(|source| DryerError::Glob {
        pattern: patterns.join(", "),
        source,
    })
}
