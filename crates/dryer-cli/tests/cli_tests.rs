use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn cli_prints_text_candidates() {
    let output = Command::new(env!("CARGO_BIN_EXE_dryer"))
        .arg(fixture_path("rust/renamed_function_duplicate"))
        .output()
        .expect("run dryer");

    assert!(output.status.success(), "{output:#?}");
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("DUPLICATE rust score=1.00"), "{stdout}");
    assert!(stdout.contains("invoice_summary"), "{stdout}");
    assert!(stdout.contains("receipt_summary"), "{stdout}");
}

#[test]
fn cli_returns_one_when_fail_on_duplicates_finds_candidates() {
    let output = Command::new(env!("CARGO_BIN_EXE_dryer"))
        .arg("--fail-on-duplicates")
        .arg(fixture_path("rust/renamed_function_duplicate"))
        .output()
        .expect("run dryer");

    assert_eq!(Some(1), output.status.code(), "{output:#?}");
}

#[test]
fn cli_accepts_typescript_language_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_dryer"))
        .arg("--language")
        .arg("typescript")
        .arg("--json")
        .arg(fixture_path("typescript/react_component_duplicate"))
        .output()
        .expect("run dryer");

    assert!(output.status.success(), "{output:#?}");
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("\"language\": \"typescript\""), "{stdout}");
}

#[test]
fn cli_rejects_invalid_language_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_dryer"))
        .arg("--language")
        .arg("python")
        .arg(fixture_path("rust/renamed_function_duplicate"))
        .output()
        .expect("run dryer");

    assert_eq!(Some(2), output.status.code(), "{output:#?}");
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
