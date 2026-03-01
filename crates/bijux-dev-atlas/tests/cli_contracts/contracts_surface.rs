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

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

#[test]
fn contracts_ops_list_includes_tests_by_default() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--list", "--format", "json"])
        .output()
        .expect("contracts ops list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let contracts = payload["contracts"].as_array().expect("contracts array");
    assert!(!contracts.is_empty());
    let tests = contracts[0]["tests"].as_array().expect("tests array");
    assert!(!tests.is_empty());
}

#[test]
fn contracts_ops_list_contains_curated_contract_ids() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--list", "--format", "json"])
        .output()
        .expect("contracts ops list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let ids = payload["contracts"]
        .as_array()
        .expect("contracts array")
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for expected in [
        "OPS-ROOT-001",
        "OPS-ROOT-002",
        "OPS-ROOT-015",
        "OPS-ROOT-021",
        "OPS-ROOT-023",
        "OPS-INV-001",
        "OPS-SCHEMA-001",
        "OPS-STACK-001",
    ] {
        assert!(
            ids.contains(expected),
            "missing curated contract id {expected}"
        );
    }
}

#[test]
fn contracts_ops_list_keeps_root_markdown_contract_tests_stable() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--list", "--format", "json"])
        .output()
        .expect("contracts ops list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let contracts = payload["contracts"].as_array().expect("contracts array");
    for (expected, expected_tests) in [
        ("OPS-ROOT-002", vec!["ops.root.forbid_extra_markdown"]),
        ("OPS-ROOT-015", vec!["ops.root.no_extra_pillar_markdown"]),
    ] {
        let row = contracts
            .iter()
            .find(|row| row["id"].as_str() == Some(expected))
            .unwrap_or_else(|| panic!("missing ops contract {expected}"));
        let tests = row["tests"].as_array().expect("tests array");
        let actual = tests
            .iter()
            .filter_map(|test| test["test_id"].as_str())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected_tests, "unexpected tests for {expected}");
    }
}

#[test]
#[ignore = "legacy docs contracts topology"]
fn contracts_ops_supports_junit_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--format", "junit"])
        .output()
        .expect("contracts ops junit");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(text.contains("<testsuite"));
    assert!(text.contains("contracts.ops"));
}

#[test]
fn contracts_all_lists_all_domains() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "all", "--list", "--format", "json"])
        .output()
        .expect("contracts all list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let domains = payload["contracts"]
        .as_array()
        .expect("contracts array")
        .iter()
        .filter_map(|row| row["domain"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(domains.contains("configs"));
    assert!(domains.contains("control-plane"));
    assert!(domains.contains("docs"));
    assert!(domains.contains("docker"));
    assert!(domains.contains("make"));
    assert!(domains.contains("ops"));
    assert!(domains.contains("root"));
    assert!(domains.contains("runtime"));
}

#[test]
fn contracts_all_list_includes_severity_column() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "all", "--list", "--format", "json"])
        .output()
        .expect("contracts all list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let row = payload["contracts"]
        .as_array()
        .expect("contracts array")
        .first()
        .expect("first row");
    assert_eq!(row["severity"].as_str(), Some("must"));
}

#[test]
fn contracts_all_list_is_sorted_by_domain_then_id() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "all", "--list", "--format", "json"])
        .output()
        .expect("contracts all list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let rows = payload["contracts"].as_array().expect("contracts array");
    let mut actual = rows
        .iter()
        .map(|row| {
            (
                row["domain"].as_str().expect("domain").to_string(),
                row["id"].as_str().expect("id").to_string(),
            )
        })
        .collect::<Vec<_>>();
    let mut sorted = actual.clone();
    sorted.sort();
    assert_eq!(actual, sorted, "contracts --list must stay sorted");
    actual.dedup();
    assert_eq!(actual.len(), rows.len(), "contracts list must not include duplicates");
}

#[test]
fn contracts_all_list_json_is_deterministic() {
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
            .current_dir(repo_root())
            .args(["contracts", "all", "--list", "--format", "json"])
            .output()
            .expect("contracts all list")
    };
    let first = run();
    let second = run();
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout, "contracts list output must be deterministic");
}

#[test]
fn contracts_all_json_domain_order_is_stable() {
    let source = fs::read_to_string(
        repo_root().join("crates/bijux-dev-atlas/src/commands/control_plane_contracts.rs"),
    )
    .expect("read contracts command source");
    assert!(
        source.contains(
            "vec![\n                    \"root\",\n                    \"runtime\",\n                    \"control-plane\",\n                    \"docker\",\n                    \"make\",\n                    \"ops\",\n                    \"configs\",\n                    \"docs\",\n                ]"
        ),
        "contracts all domain order changed"
    );
}

#[test]
fn contracts_runtime_list_exposes_runtime_registry() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "runtime", "--list", "--format", "json"])
        .output()
        .expect("contracts runtime list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let ids = payload["contracts"]
        .as_array()
        .expect("contracts array")
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(ids.contains("RUNTIME-001"));
    assert!(ids.contains("RUNTIME-005"));
}

#[test]
fn contracts_control_plane_list_exposes_control_plane_registry() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "control-plane", "--list", "--format", "json"])
        .output()
        .expect("contracts control-plane list");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let ids = payload["contracts"]
        .as_array()
        .expect("contracts array")
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(ids.contains("CONTROL-PLANE-001"));
    assert!(ids.contains("CONTROL-PLANE-005"));
}

#[test]
fn contracts_run_id_override_reaches_report_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "runtime",
            "--mode",
            "static",
            "--format",
            "json",
            "--run-id",
            "runtime_contracts_smoke",
        ])
        .output()
        .expect("contracts runtime json");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload["run_id"].as_str(),
        Some("runtime_contracts_smoke"),
        "contracts run_id override must reach report metadata"
    );
}

#[test]
fn contracts_docs_list_json_matches_golden_snapshot() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docs", "--list", "--format", "json"])
        .output()
        .expect("contracts docs list json");
    assert!(output.status.success());
    let actual = String::from_utf8(output.stdout).expect("utf8 stdout");
    let golden = fs::read_to_string(
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/contracts_docs_list.json"),
    )
    .expect("read contracts docs list golden");
    assert_eq!(normalize_newlines(&actual), normalize_newlines(&golden));
}

#[test]
fn contracts_docs_list_table_matches_golden_snapshot() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docs", "--list", "--format", "table"])
        .output()
        .expect("contracts docs list table");
    assert!(output.status.success());
    let actual = String::from_utf8(output.stdout).expect("utf8 stdout");
    let golden = fs::read_to_string(
        repo_root().join("crates/bijux-dev-atlas/tests/goldens/contracts_docs_list_table.txt"),
    )
    .expect("read contracts docs table golden");
    assert_eq!(normalize_newlines(&actual), normalize_newlines(&golden));
}

#[test]
fn contracts_changed_only_uses_merge_base_diff_selection() {
    let source = fs::read_to_string(
        repo_root().join("crates/bijux-dev-atlas/src/commands/control_plane_contracts.rs"),
    )
    .expect("read contracts command source");
    assert!(
        source.contains(r#""merge-base", "HEAD""#),
        "changed-only must use merge-base selection"
    );
    assert!(
        source.contains(r#""diff", "--name-only""#),
        "changed-only must use git diff --name-only from merge-base"
    );
}

#[test]
fn contracts_changed_only_reports_fallback_reason_when_merge_base_unavailable() {
    let source = fs::read_to_string(
        repo_root().join("crates/bijux-dev-atlas/src/commands/control_plane_contracts.rs"),
    )
    .expect("read contracts command source");
    assert!(
        source.contains("changed-only merge-base unavailable; selected by fallback"),
        "changed-only fallback reason changed"
    );
    assert!(
        source.contains(
            "Changed-only selection note: merge-base diff could not be resolved; selecting requested domains"
        ),
        "changed-only fallback selection note changed"
    );
}

#[test]
fn contracts_invalid_contract_filter_pattern_is_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docs", "--filter-contract", "DOC-[001"])
        .output()
        .expect("contracts invalid contract filter");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("invalid wildcard pattern `DOC-[001`"));
    assert!(stderr.contains("use `*` and `?` only"));
}

#[test]
fn contracts_invalid_test_filter_pattern_is_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docs", "--filter-test", "docs.{broken}"])
        .output()
        .expect("contracts invalid test filter");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("invalid wildcard pattern `docs.{broken}`"));
}

#[test]
#[ignore = "legacy docs contracts topology"]
fn contracts_ci_human_output_disables_ansi_color() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docs", "--ci"])
        .output()
        .expect("contracts docs ci");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(!stdout.contains("\u{1b}["));
}

#[test]
fn contracts_configs_runs_and_reports_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "configs", "--format", "json"])
        .output()
        .expect("contracts configs");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["domain"].as_str(), Some("configs"));
    assert_eq!(payload["group"].as_str(), Some("configs"));
    assert_eq!(payload["summary"]["fail"].as_u64(), Some(0));
}

#[test]
#[ignore = "legacy docs contracts topology"]
fn contracts_docs_runs_and_reports_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docs", "--format", "json"])
        .output()
        .expect("contracts docs");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["domain"].as_str(), Some("docs"));
    assert_eq!(payload["summary"]["fail"].as_u64(), Some(0));
}

#[test]
#[ignore = "legacy docs contracts topology"]
fn contracts_docs_writes_report_artifacts() {
    let artifacts_root = repo_root().join("artifacts/tests/contracts-docs-report");
    fs::create_dir_all(&artifacts_root).expect("mkdir artifacts");
    let report_paths = [
        "docs-index-correctness.json",
        "broken-links.json",
        "orphans.json",
        "metadata-coverage.json",
        "duplication-report.json",
        "coverage-report.json",
    ]
    .iter()
    .map(|name| artifacts_root.join(name))
    .collect::<Vec<_>>();
    for report_path in &report_paths {
        if report_path.exists() {
            fs::remove_file(report_path).expect("remove prior report");
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "docs",
            "--format",
            "json",
            "--artifacts-root",
            artifacts_root.to_str().expect("artifacts root"),
        ])
        .output()
        .expect("contracts docs report");
    assert!(output.status.success());
    let kinds = [
        ("docs-index-correctness.json", "docs_index_correctness"),
        ("broken-links.json", "docs_broken_links"),
        ("orphans.json", "docs_orphans"),
        ("metadata-coverage.json", "docs_metadata_coverage"),
        ("duplication-report.json", "docs_duplication"),
        ("coverage-report.json", "docs_contract_coverage"),
    ];
    for (file_name, expected_kind) in kinds {
        let path = artifacts_root.join(file_name);
        let payload: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).expect("read report"))
                .expect("report json");
        assert_eq!(payload["kind"].as_str(), Some(expected_kind));
        if file_name == "duplication-report.json" {
            assert!(
                payload["analyzed_pairs"].as_array().is_some_and(|rows| !rows.is_empty()),
                "duplication report must include analyzed similarity pairs"
            );
            assert!(
                matches!(payload["status"].as_str(), Some("pass" | "warn")),
                "duplication report must emit a stable status"
            );
        }
    }
}

include!("contracts_surface_effects.rs");
