use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dryer_core::{
    find_duplicates, format_json, format_sarif, format_text, Config, LanguageFilter, NameMode,
    OutputFormat,
};

#[test]
fn rust_functions_match_after_renaming() {
    let report = run_fixture("rust/renamed_function_duplicate", LanguageFilter::Rust);

    assert_pair_named(&report, "invoice_summary", "receipt_summary");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn rust_impl_methods_match_after_renaming() {
    let report = run_fixture("rust/impl_method_duplicate", LanguageFilter::Rust);

    assert_pair_named(&report, "render", "draw");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn rust_trait_default_methods_match_after_renaming() {
    let report = run_fixture("rust/trait_default_method_duplicate", LanguageFilter::Rust);

    assert_pair_named(&report, "format_invoice", "format_receipt");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn rust_different_control_flow_does_not_match() {
    let report = run_fixture("rust/negative_different_control_flow", LanguageFilter::Rust);

    assert!(report.candidates.is_empty(), "{report:#?}");
}

#[test]
fn rust_tiny_getters_are_filtered() {
    let report = run_fixture("rust/tiny_getters_filtered", LanguageFilter::Rust);

    assert!(report.candidates.is_empty(), "{report:#?}");
}

#[test]
fn typescript_functions_match_after_renaming() {
    let report = run_fixture(
        "typescript/renamed_function_duplicate",
        LanguageFilter::TypeScript,
    );

    assert_pair_named(&report, "buildOrderRows", "buildInvoiceRows");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn typescript_arrow_functions_match_after_renaming() {
    let report = run_fixture(
        "typescript/arrow_function_duplicate",
        LanguageFilter::TypeScript,
    );

    assert_pair_named(&report, "summarizeOrders", "summarizeInvoices");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn typescript_object_methods_match_after_renaming() {
    let report = run_fixture(
        "typescript/object_method_duplicate",
        LanguageFilter::TypeScript,
    );

    assert_pair_named(&report, "buildRows", "buildLines");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn typescript_class_methods_match_after_renaming() {
    let report = run_fixture(
        "typescript/class_method_duplicate",
        LanguageFilter::TypeScript,
    );

    assert_pair_named(&report, "buildRows", "buildLines");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn tsx_components_match_after_renaming() {
    let report = run_fixture(
        "typescript/react_component_duplicate",
        LanguageFilter::TypeScript,
    );

    assert_pair_named(&report, "OrderPanel", "InvoicePanel");
    assert!(report.candidates[0].score >= 0.82);
}

#[test]
fn typescript_different_control_flow_does_not_match() {
    let report = run_fixture(
        "typescript/negative_different_control_flow",
        LanguageFilter::TypeScript,
    );

    assert!(report.candidates.is_empty(), "{report:#?}");
}

#[test]
fn typescript_tiny_getters_are_filtered() {
    let report = run_fixture(
        "typescript/tiny_getters_filtered",
        LanguageFilter::TypeScript,
    );

    assert!(report.candidates.is_empty(), "{report:#?}");
}

#[test]
fn declaration_files_are_ignored_by_default() {
    let report = run_fixture("typescript/dts_ignored", LanguageFilter::TypeScript);

    assert!(report.candidates.is_empty(), "{report:#?}");
}

#[test]
fn generated_files_are_excluded_by_default() {
    let report = run_fixture("mixed/generated_excludes", LanguageFilter::All);

    assert!(report.candidates.is_empty(), "{report:#?}");
}

#[test]
fn mixed_scan_finds_rust_and_typescript_pairs() {
    let report = run_fixture("mixed/rust_and_typescript", LanguageFilter::All);

    assert_pair_named(&report, "build_order_rows", "build_invoice_rows");
    assert_pair_named(&report, "buildOrderRows", "buildInvoiceRows");
    assert!(report
        .candidates
        .iter()
        .any(|candidate| candidate.language == "rust"));
    assert!(report
        .candidates
        .iter()
        .any(|candidate| candidate.language == "typescript"));
}

#[test]
fn text_output_is_stable_for_no_candidates() {
    let report = run_fixture("rust/tiny_getters_filtered", LanguageFilter::Rust);

    assert_eq!("No duplicate candidates found.\n", format_text(&report));
}

#[test]
fn json_output_uses_report_schema() {
    let report = run_fixture("rust/renamed_function_duplicate", LanguageFilter::Rust);
    let json = format_json(&report).expect("json output");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

    assert!(value.get("candidates").is_some());
    assert_eq!(
        value["candidates"][0]["language"],
        serde_json::Value::String("rust".to_string())
    );
}

#[test]
fn sarif_output_uses_sarif_schema() {
    let report = run_fixture("rust/renamed_function_duplicate", LanguageFilter::Rust);
    let sarif = format_sarif(&report).expect("sarif output");
    let value: serde_json::Value = serde_json::from_str(&sarif).expect("valid json");

    assert_eq!(
        value["version"],
        serde_json::Value::String("2.1.0".to_string())
    );
    assert_eq!(
        value["runs"][0]["tool"]["driver"]["name"],
        serde_json::Value::String("dryer".to_string())
    );
    assert!(value["runs"][0]["results"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
}

#[test]
fn config_file_parses_documented_options() {
    let path = temp_config_path();
    fs::write(
        &path,
        r#"
threshold = 0.9
min_lines = 5
min_nodes = 25
language = "typescript"
format = "json"
cross_language = true
max_candidates = 3
include = ["**/*.ts"]
exclude = ["**/*.generated.ts"]

[normalization]
name_mode = "strict"
preserve_call_names = true
preserve_property_names = true
preserve_type_names = true
"#,
    )
    .expect("write config");

    let config = Config::from_file(&path).expect("config loads");

    assert_eq!(0.9, config.threshold);
    assert_eq!(5, config.min_lines);
    assert_eq!(25, config.min_nodes);
    assert_eq!(LanguageFilter::TypeScript, config.language);
    assert_eq!(OutputFormat::Json, config.format);
    assert!(config.cross_language);
    assert_eq!(Some(3), config.max_candidates);
    assert_eq!(vec!["**/*.ts"], config.include);
    assert_eq!(vec!["**/*.generated.ts"], config.exclude);
    assert_eq!(NameMode::Strict, config.normalization.name_mode);

    fs::remove_file(path).ok();
}

fn run_fixture(relative: &str, language: LanguageFilter) -> dryer_core::Report {
    let path = fixture_path(relative);
    let config = Config {
        paths: vec![path],
        threshold: 0.82,
        min_lines: 4,
        min_nodes: 20,
        language,
        ..Config::default()
    };

    find_duplicates(&config).expect("fixture scan succeeds")
}

fn assert_pair_named(report: &dryer_core::Report, left: &str, right: &str) {
    assert!(!report.candidates.is_empty(), "{report:#?}");
    let found = report.candidates.iter().any(|candidate| {
        candidate.left.name.as_deref() == Some(left)
            && candidate.right.name.as_deref() == Some(right)
    });
    assert!(found, "{left} -> {right} not found in {report:#?}");
}

fn fixture_path(relative: &str) -> PathBuf {
    workspace_root().join("fixtures").join(relative)
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn temp_config_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("dryer-config-{nonce}.toml"))
}
