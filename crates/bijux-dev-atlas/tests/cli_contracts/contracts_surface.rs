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
    assert!(domains.contains("docs"));
    assert!(domains.contains("docker"));
    assert!(domains.contains("make"));
    assert!(domains.contains("ops"));
    assert!(domains.contains("root"));
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
        source.contains(r#"vec!["root", "docker", "make", "ops", "configs", "docs"]"#),
        "contracts all domain order changed"
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

#[test]
#[ignore = "legacy docs contracts topology"]
fn contracts_ops_supports_table_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--format", "table"])
        .output()
        .expect("contracts ops table");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(text.contains("CONTRACT_ID | REQUIRED | STATUS | TESTS | SUMMARY"));
}

#[test]
fn contracts_make_supports_table_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "make", "--format", "table"])
        .output()
        .expect("contracts make table");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(text.contains("CONTRACT_ID | REQUIRED | STATUS | TESTS | SUMMARY"));
}

#[test]
fn contracts_all_list_reports_required_lanes() {
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
        .iter()
        .find(|row| row["id"].as_str() == Some("ROOT-042"))
        .expect("required root contract row");
    assert_eq!(row["required"].as_bool(), Some(true));
    assert_eq!(
        row["lanes"]
            .as_array()
            .expect("lanes array")
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>(),
        vec!["pr", "merge", "release"]
    );
}

#[test]
fn contracts_required_flag_filters_to_required_contracts() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "all", "--required", "--lane", "pr", "--format", "json"])
        .output()
        .expect("contracts required only");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let domains = payload["domains"].as_array().expect("domains array");
    let all_contracts = domains
        .iter()
        .flat_map(|domain| {
            domain["contracts"]
                .as_array()
                .into_iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert!(!all_contracts.is_empty());
    assert!(all_contracts
        .iter()
        .all(|row| row["required"].as_bool() == Some(true)));
}

#[test]
fn contracts_json_includes_lane_metadata_and_required_artifact() {
    let required_artifact = repo_root().join("artifacts/contracts/required.json");
    if required_artifact.exists() {
        fs::remove_file(&required_artifact).expect("remove prior required artifact");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "root", "--lane", "pr", "--format", "json"])
        .output()
        .expect("contracts root pr lane");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["lane"].as_str(), Some("pr"));
    let required_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(required_artifact).expect("required artifact"))
            .expect("required artifact json");
    assert!(required_payload["contracts"].is_array());
}

#[test]
fn contracts_make_runs_and_reports_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "make", "--format", "json"])
        .output()
        .expect("contracts make");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["domain"].as_str(), Some("make"));
    assert_eq!(payload["summary"]["fail"].as_u64(), Some(0));
}

#[test]
fn contracts_snapshot_writes_ops_registry_file() {
    let out = repo_root().join("artifacts/tests/contracts-ops-snapshot.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "snapshot",
            "--domain",
            "ops",
            "--out",
            out.to_str().expect("out path"),
        ])
        .output()
        .expect("contracts snapshot");
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read out file");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert_eq!(payload["domain"].as_str(), Some("ops"));
    assert!(payload["contracts"].is_array());
}

#[test]
fn contracts_snapshot_defaults_to_artifacts_root() {
    let out = repo_root().join("artifacts/contracts/docker/registry-snapshot.json");
    if out.exists() {
        fs::remove_file(&out).expect("remove prior snapshot");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "snapshot", "--domain", "docker"])
        .output()
        .expect("contracts snapshot default path");
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read snapshot");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert_eq!(payload["domain"].as_str(), Some("docker"));
    assert!(payload["contracts"].is_array());
}

#[test]
fn contracts_ops_supports_filter_contract_alias() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--format",
            "json",
            "--filter-contract",
            "OPS-ROOT-017",
        ])
        .output()
        .expect("contracts ops filter-contract");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    let contracts = payload["contracts"].as_array().expect("contracts array");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0]["id"].as_str(), Some("OPS-ROOT-017"));
}

#[test]
fn contracts_ops_explain_includes_mapped_gate() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--explain",
            "OPS-ROOT-017",
            "--format",
            "json",
        ])
        .output()
        .expect("contracts ops explain");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert!(payload["mapped_gate"].as_str().is_some());
}

#[test]
fn contracts_ops_explain_test_reports_effects_and_io() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--explain-test",
            "ops.root_surface.required_commands_exist",
            "--format",
            "json",
        ])
        .output()
        .expect("contracts ops explain-test");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(
        payload["test_id"].as_str(),
        Some("ops.root_surface.required_commands_exist")
    );
    assert!(payload["inputs_read"].as_array().is_some());
    assert!(payload["outputs_written"].as_array().is_some());
    assert!(payload["effects_required"].as_array().is_some());
}

#[test]
fn contracts_docker_supports_json_and_junit_sidecar_outputs() {
    let json_out = repo_root().join("artifacts/tests/contracts-docker-report.json");
    let junit_out = repo_root().join("artifacts/tests/contracts-docker-report.xml");
    if let Some(parent) = json_out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "docker",
            "--format",
            "human",
            "--json-out",
            json_out.to_str().expect("json out"),
            "--junit-out",
            junit_out.to_str().expect("junit out"),
        ])
        .output()
        .expect("contracts docker sidecar outputs");
    assert!(output.status.success());
    let human = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(human.contains("Contracts: docker"));
    let json_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(json_out).expect("json out"))
            .expect("json report");
    assert_eq!(json_payload["domain"].as_str(), Some("docker"));
    let junit_text = fs::read_to_string(junit_out).expect("junit out");
    assert!(junit_text.contains("<testsuite"));
}

#[test]
fn contracts_ops_effect_mode_requires_explicit_allow_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--mode", "effect", "--format", "json"])
        .output()
        .expect("contracts ops effect mode");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("effect mode requires"));
    assert!(stderr.contains("--allow-subprocess"));
    assert!(stderr.contains("--allow-network"));
}

#[test]
#[ignore = "legacy docs contracts topology"]
fn contracts_ops_ci_uses_default_artifacts_root() {
    let out = repo_root().join("artifacts/contracts/ops/local/static/local/ops.json");
    let inventory = repo_root().join("artifacts/contracts/ops/local/static/local/ops.inventory.json");
    let maturity = repo_root().join("artifacts/contracts/ops/local/static/local/ops.maturity.json");
    if out.exists() {
        fs::remove_file(&out).expect("remove prior report");
    }
    if inventory.exists() {
        fs::remove_file(&inventory).expect("remove prior inventory");
    }
    if maturity.exists() {
        fs::remove_file(&maturity).expect("remove prior maturity");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .env("CI", "true")
        .args(["contracts", "ops", "--format", "json"])
        .output()
        .expect("contracts ops ci");
    assert!(output.status.success());
    let written = fs::read_to_string(out).expect("read generated default report");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("json file");
    assert_eq!(payload["domain"].as_str(), Some("ops"));
    let inventory_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(inventory).expect("inventory file"))
            .expect("inventory json");
    assert_eq!(inventory_payload["domain"].as_str(), Some("ops"));
    let maturity_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(maturity).expect("maturity file"))
            .expect("maturity json");
    assert_eq!(maturity_payload["domain"].as_str(), Some("ops"));
    assert!(maturity_payload["maturity"].is_object());
}

#[test]
fn contracts_profile_changes_default_artifacts_root_segment() {
    let out = repo_root().join("artifacts/contracts/docker/ci/static/local/docker.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    if out.exists() {
        fs::remove_file(&out).expect("remove prior report");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docker", "--profile", "ci", "--format", "json"])
        .output()
        .expect("contracts docker with profile");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out).expect("read generated report"))
            .expect("json report");
    assert_eq!(payload["domain"].as_str(), Some("docker"));
}

#[test]
fn contracts_docker_effect_requires_only_selected_effect_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "docker",
            "--mode",
            "effect",
            "--filter-contract",
            "DOCKER-100",
        ])
        .output()
        .expect("contracts docker effect mode");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("--allow-subprocess"));
    assert!(!stderr.contains("--allow-network"));
}

#[test]
fn contracts_ci_forbids_skip_without_explicit_override() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .env("CI", "true")
        .args([
            "contracts",
            "ops",
            "--skip",
            "OPS-ROOT-*",
            "--artifacts-root",
            "artifacts/tests/contracts-ci-skip",
        ])
        .output()
        .expect("contracts ops ci skip");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("forbid --skip"));
}

#[test]
fn contracts_ops_json_report_matches_schema() {
    let artifacts_root = repo_root().join("artifacts/tests/contracts-json-schema");
    fs::create_dir_all(&artifacts_root).expect("mkdir artifacts");
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            "ops",
            "--mode",
            "static",
            "--filter-contract",
            "OPS-ROOT-001",
            "--format",
            "json",
            "--artifacts-root",
            artifacts_root.to_str().expect("artifacts root"),
        ])
        .output()
        .expect("contracts ops json report");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["schema_version"].as_u64(), Some(1));
    assert_eq!(payload["domain"].as_str(), Some("ops"));
    assert_eq!(payload["mode"].as_str(), Some("static"));
    assert!(payload["summary"]["contracts"].as_u64().is_some());
    assert!(payload["summary"]["tests"].as_u64().is_some());
    let contracts = payload["contracts"].as_array().expect("contracts array");
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0]["id"].as_str(), Some("OPS-ROOT-001"));
    assert!(matches!(
        contracts[0]["status"].as_str(),
        Some("PASS" | "FAIL" | "SKIP" | "ERROR")
    ));
    let tests = payload["tests"].as_array().expect("tests array");
    assert!(!tests.is_empty());
    for case in tests {
        assert!(case["contract_id"].as_str().is_some());
        assert!(case["contract_title"].as_str().is_some());
        assert!(case["test_id"].as_str().is_some());
        assert!(case["test_title"].as_str().is_some());
        assert!(matches!(
            case["kind"].as_str(),
            Some("pure" | "subprocess" | "network")
        ));
        assert!(matches!(
            case["status"].as_str(),
            Some("PASS" | "FAIL" | "SKIP" | "ERROR")
        ));
        assert!(case["note"].is_string() || case["note"].is_null());
        assert!(case["violations"].as_array().is_some());
    }
}

#[test]
#[ignore = "legacy docs contracts topology"]
fn contracts_ops_changed_only_runs_and_reports_ops_domain() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "ops", "--changed-only", "--format", "json"])
        .output()
        .expect("contracts ops changed-only");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid json output");
    assert_eq!(payload["domain"].as_str(), Some("ops"));
    assert_eq!(payload["group"].as_str(), Some("ops"));
    assert!(payload["summary"]["contracts"].as_u64().is_some());
}
