#[test]
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
    let required_artifact = repo_root().join("ops/_generated.example/contracts-required.json");
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
    let out = repo_root().join("artifacts/run/local/gates/contracts/ops/local/static/ops.json");
    let inventory =
        repo_root().join("artifacts/run/local/gates/contracts/ops/local/static/ops.inventory.json");
    let maturity =
        repo_root().join("artifacts/run/local/gates/contracts/ops/local/static/ops.maturity.json");
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
    fn collect_candidates(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_candidates(&path, out);
                continue;
            }
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "docker.json")
            {
                out.push(path);
            }
        }
    }

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", "docker", "--profile", "ci", "--format", "json"])
        .output()
        .expect("contracts docker with profile");
    assert!(output.status.success());
    let mut candidates = Vec::new();
    collect_candidates(&repo_root().join("artifacts"), &mut candidates);
    candidates.sort();
    let out = candidates
        .into_iter()
        .find(|path| {
            let rel = path.to_string_lossy();
            rel.contains("/docker/")
                && rel.contains("/ci/")
                && rel.ends_with("/docker.json")
        })
        .expect("docker profile report path");
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
