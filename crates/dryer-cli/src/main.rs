use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use dryer_core::{
    find_duplicates, format_json, format_sarif, format_text, Config, LanguageFilter, OutputFormat,
};

#[derive(Debug, Parser)]
#[command(name = "dryer")]
#[command(
    about = "Find structural duplicate candidates in Rust, TypeScript, Haskell, and Daml code"
)]
struct Args {
    #[arg(value_name = "FILE_OR_DIRECTORY")]
    paths: Vec<PathBuf>,

    #[arg(long)]
    threshold: Option<f64>,

    #[arg(long = "min-lines")]
    min_lines: Option<usize>,

    #[arg(long = "min-nodes")]
    min_nodes: Option<usize>,

    #[arg(long, value_enum)]
    language: Option<CliLanguage>,

    #[arg(long, value_enum)]
    format: Option<CliFormat>,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    sarif: bool,

    #[arg(long = "include")]
    include: Vec<String>,

    #[arg(long = "exclude")]
    exclude: Vec<String>,

    #[arg(long = "cross-language")]
    cross_language: bool,

    #[arg(long = "max-candidates")]
    max_candidates: Option<usize>,

    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long = "fail-on-duplicates")]
    fail_on_duplicates: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliLanguage {
    All,
    Rust,
    #[value(name = "typescript")]
    TypeScript,
    Haskell,
    Daml,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliFormat {
    Text,
    Json,
    Sarif,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = build_config(&args)?;
    let report = find_duplicates(&config)?;

    match config.format {
        OutputFormat::Text => print!("{}", format_text(&report)),
        OutputFormat::Json => println!("{}", format_json(&report)?),
        OutputFormat::Sarif => println!("{}", format_sarif(&report)?),
    }

    if args.fail_on_duplicates && !report.candidates.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

fn build_config(args: &Args) -> Result<Config> {
    let mut config = match &args.config {
        Some(path) => {
            Config::from_file(path).with_context(|| format!("failed to load {}", path.display()))?
        }
        None if PathBuf::from("dryer.toml").is_file() => {
            Config::from_file(&PathBuf::from("dryer.toml")).context("failed to load dryer.toml")?
        }
        None => Config::default(),
    };

    if !args.paths.is_empty() {
        config.paths = args.paths.clone();
    }
    if let Some(threshold) = args.threshold {
        config.threshold = threshold;
    }
    if let Some(min_lines) = args.min_lines {
        config.min_lines = min_lines;
    }
    if let Some(min_nodes) = args.min_nodes {
        config.min_nodes = min_nodes;
    }
    if let Some(language) = args.language {
        config.language = match language {
            CliLanguage::All => LanguageFilter::All,
            CliLanguage::Rust => LanguageFilter::Rust,
            CliLanguage::TypeScript => LanguageFilter::TypeScript,
            CliLanguage::Haskell => LanguageFilter::Haskell,
            CliLanguage::Daml => LanguageFilter::Daml,
        };
    }
    if let Some(format) = args.format {
        config.format = match format {
            CliFormat::Text => OutputFormat::Text,
            CliFormat::Json => OutputFormat::Json,
            CliFormat::Sarif => OutputFormat::Sarif,
        };
    }
    if args.json {
        config.format = OutputFormat::Json;
    }
    if args.sarif {
        config.format = OutputFormat::Sarif;
    }
    if !args.include.is_empty() {
        config.include = args.include.clone();
    }
    if !args.exclude.is_empty() {
        config.exclude.extend(args.exclude.clone());
    }
    if args.cross_language {
        config.cross_language = true;
    }
    if let Some(max_candidates) = args.max_candidates {
        config.max_candidates = Some(max_candidates);
    }

    Ok(config)
}
