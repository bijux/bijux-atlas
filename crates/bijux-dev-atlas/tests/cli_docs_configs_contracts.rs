// SPDX-License-Identifier: Apache-2.0

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

#[test]
fn slow_docs_validate_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "validate", "--format", "json"])
        .output()
        .expect("docs validate json");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload.get("errors").and_then(|v| v.as_array()).is_some());
}

#[test]
fn docs_check_requires_allow_subprocess() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "check", "--format", "json"])
        .output()
        .expect("docs check");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docs check requires --allow-subprocess"));
}

#[test]
fn docs_serve_requires_allow_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "serve", "--allow-subprocess", "--format", "json"])
        .output()
        .expect("docs serve");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docs serve requires --allow-network"));
}

#[test]
fn docs_clean_requires_allow_write() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "clean", "--format", "json"])
        .output()
        .expect("docs clean");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("docs clean requires --allow-write"));
}

#[test]
fn docs_inventory_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "inventory", "--format", "json"])
        .output()
        .expect("docs inventory json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("pages").and_then(|v| v.as_array()).is_some());
    assert!(payload.get("nav").and_then(|v| v.as_array()).is_some());
}

#[test]
fn docs_verify_contracts_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "verify-contracts", "--format", "json"])
        .output()
        .expect("docs verify-contracts json");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload.get("summary").is_some());
}

#[test]
fn docs_inventory_respects_include_drafts_flag() {
    let fixture_root = repo_root().join("crates/bijux-dev-atlas/tests/fixtures/docs-mini");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture_root)
        .args([
            "docs",
            "inventory",
            "--repo-root",
            ".",
            "--include-drafts",
            "--format",
            "json",
        ])
        .output()
        .expect("docs inventory include drafts");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let pages = payload
        .get("pages")
        .and_then(|v| v.as_array())
        .expect("pages");
    assert!(pages.iter().any(|row| {
        row.get("path")
            .and_then(|v| v.as_str())
            .is_some_and(|p| p == "_drafts/draft.md")
    }));
}

#[test]
fn docs_inventory_fixture_json_matches_golden() {
    let fixture_root = repo_root().join("crates/bijux-dev-atlas/tests/fixtures/docs-mini");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(&fixture_root)
        .args([
            "docs",
            "inventory",
            "--repo-root",
            ".",
            "--run-id",
            "docs_inventory_fixture",
            "--format",
            "json",
        ])
        .output()
        .expect("docs inventory fixture");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let mut payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    payload["duration_ms"] = serde_json::json!(0);
    let actual = serde_json::to_string_pretty(&payload).expect("json");
    let golden_path =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/docs_inventory_fixture.json");
    let golden = fs::read_to_string(golden_path).expect("golden");
    assert_eq!(actual.trim(), golden.trim());
}

#[test]
fn slow_docs_validate_strict_escalates_warnings_to_errors() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "validate", "--strict", "--format", "json"])
        .output()
        .expect("docs validate strict");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload
            .get("options")
            .and_then(|v| v.get("strict"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(payload.get("error_code").is_some());
}

#[test]
fn docs_links_strict_escalates_generated_link_warnings() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "links", "--strict", "--format", "json"])
        .output()
        .expect("docs links strict");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(
        payload
            .get("options")
            .and_then(|v| v.get("strict"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(payload
        .get("errors")
        .and_then(|v| v.as_array())
        .is_some_and(|rows| !rows.is_empty()));
}

#[test]
fn docs_doctor_fixture_json_matches_golden() {
    let fixture_root = repo_root().join("crates/bijux-dev-atlas/tests/fixtures/docs-mini");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "docs",
            "doctor",
            "--repo-root",
            fixture_root.to_str().expect("fixture root"),
            "--run-id",
            "docs_fixture",
            "--format",
            "json",
        ])
        .output()
        .expect("docs doctor fixture");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let mut payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    payload["duration_ms"] = serde_json::json!(0);
    let actual = serde_json::to_string_pretty(&payload).expect("json");
    let golden_path =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/docs_doctor_fixture.json");
    let golden = fs::read_to_string(golden_path).expect("golden");
    assert_eq!(actual.trim(), golden.trim());
}

#[test]
fn slow_docs_registry_build_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "docs",
            "registry",
            "build",
            "--allow-write",
            "--format",
            "json",
        ])
        .output()
        .expect("docs registry build");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload.get("artifacts").is_some());
    assert!(payload
        .get("artifacts")
        .and_then(|v| v.get("docs_test_coverage"))
        .and_then(|v| v.as_str())
        .is_some());
    assert!(payload
        .get("artifacts")
        .and_then(|v| v.get("docs_quality_dashboard"))
        .and_then(|v| v.as_str())
        .is_some());
    assert!(payload
        .get("artifacts")
        .and_then(|v| v.get("crate_doc_governance"))
        .and_then(|v| v.as_str())
        .is_some());
    assert!(payload
        .get("artifacts")
        .and_then(|v| v.get("crate_doc_api_table"))
        .and_then(|v| v.as_str())
        .is_some());
    assert!(payload
        .get("artifacts")
        .and_then(|v| v.get("crate_doc_pruning"))
        .and_then(|v| v.as_str())
        .is_some());
}

#[test]
fn slow_docs_registry_validate_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["docs", "registry", "validate", "--format", "json"])
        .output()
        .expect("docs registry validate");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(payload.get("summary").is_some());
}

#[test]
fn configs_inventory_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "inventory", "--format", "json"])
        .output()
        .expect("configs inventory json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn configs_list_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "list", "--format", "json"])
        .output()
        .expect("configs list json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn configs_inventory_writes_artifact_when_allow_write_enabled() {
    let artifact_root = repo_root().join("artifacts/tests/configs_inventory_write");
    if artifact_root.exists() {
        let _ = std::fs::remove_dir_all(&artifact_root);
    }
    std::fs::create_dir_all(&artifact_root).expect("mkdir");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "configs",
            "inventory",
            "--allow-write",
            "--artifacts-root",
            artifact_root.to_str().expect("artifact root"),
            "--run-id",
            "configs_inventory_test",
            "--format",
            "json",
        ])
        .output()
        .expect("configs inventory write");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let path = payload
        .get("artifacts")
        .and_then(|v| v.get("inventory"))
        .and_then(|v| v.as_str())
        .expect("inventory artifact path");
    assert!(
        std::path::Path::new(path).exists(),
        "artifact file must exist"
    );
}

#[test]
fn configs_print_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "print", "--format", "json"])
        .output()
        .expect("configs print");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload.get("rows").and_then(|v| v.as_array()).is_some());
}

#[test]
fn crate_doc_governance_snapshot_matches_golden() {
    let governance_path = repo_root().join("docs/_generated/crate-doc-governance.json");
    let governance_text = fs::read_to_string(governance_path).expect("governance json");
    let governance: serde_json::Value =
        serde_json::from_str(&governance_text).expect("valid governance json");
    let rows = governance
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "crate": row.get("crate").and_then(|v| v.as_str()).unwrap_or_default(),
                "root_doc_count": row.get("root_doc_count").and_then(|v| v.as_u64()).unwrap_or(0),
                "docs_dir_count": row.get("docs_dir_count").and_then(|v| v.as_u64()).unwrap_or(0),
                "diagram_count": row.get("diagram_count").and_then(|v| v.as_u64()).unwrap_or(0)
            })
        })
        .collect::<Vec<_>>();
    let actual = serde_json::json!({
        "schema_version": governance.get("schema_version").and_then(|v| v.as_u64()).unwrap_or(1),
        "rows": rows
    });

    let golden_path =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/crate_doc_governance_snapshot.json");
    let golden_text = fs::read_to_string(golden_path).expect("governance golden");
    let golden: serde_json::Value = serde_json::from_str(&golden_text).expect("valid golden");
    assert_eq!(actual, golden);
}

#[test]
fn configs_fmt_check_supports_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["configs", "fmt", "--check", "--format", "json"])
        .output()
        .expect("configs fmt");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    assert_eq!(payload.get("mode").and_then(|v| v.as_str()), Some("check"));
}

#[test]
fn configs_doctor_fixture_json_matches_golden() {
    let fixture_root = repo_root().join("crates/bijux-dev-atlas/tests/fixtures/configs-mini");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "configs",
            "doctor",
            "--repo-root",
            fixture_root.to_str().expect("fixture root"),
            "--run-id",
            "configs_fixture",
            "--format",
            "json",
        ])
        .output()
        .expect("configs doctor fixture");
    let bytes = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let mut payload: serde_json::Value = serde_json::from_slice(bytes).expect("valid json output");
    payload["duration_ms"] = serde_json::json!(0);
    let actual = serde_json::to_string_pretty(&payload).expect("json");
    let golden_path =
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/configs_doctor_fixture.json");
    let golden = fs::read_to_string(golden_path).expect("golden");
    assert_eq!(actual.trim(), golden.trim());
}
