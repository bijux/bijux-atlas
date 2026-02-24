use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn normalize_json(value: serde_json::Value) -> String {
    serde_json::to_string_pretty(&value).expect("json")
}

#[test]
fn ops_help_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "--help"])
        .output()
        .expect("ops help");
    assert!(output.status.success());
    let actual = String::from_utf8(output.stdout).expect("utf8");
    let golden = repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_help.txt");
    if std::env::var("UPDATE_GOLDENS").ok().as_deref() == Some("1") {
        fs::write(&golden, &actual).expect("write golden");
    }
    let expected = fs::read_to_string(golden).expect("golden");
    assert_eq!(actual, expected);
}

#[test]
fn ops_inventory_json_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "inventory", "--format", "json"])
        .output()
        .expect("ops inventory");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("json");
    let actual = normalize_json(payload);
    let golden = repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_inventory.json");
    if std::env::var("UPDATE_GOLDENS").ok().as_deref() == Some("1") {
        fs::write(&golden, &actual).expect("write golden");
    }
    let expected = fs::read_to_string(golden).expect("golden");
    assert_eq!(actual, expected);
}

#[test]
fn ops_suite_list_json_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["ops", "suite", "list", "--format", "json"])
        .output()
        .expect("ops suite list");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    let actual = normalize_json(payload);
    let golden = repo_root().join("crates/bijux-dev-atlas/tests/goldens/ops_suite_list.json");
    if std::env::var("UPDATE_GOLDENS").ok().as_deref() == Some("1") {
        fs::write(&golden, &actual).expect("write golden");
    }
    let expected = fs::read_to_string(golden).expect("golden");
    assert_eq!(actual, expected);
}
