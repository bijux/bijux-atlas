// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};
mod boundary_contract_reports;
use boundary_contract_reports::{
    run_boundary_helm_env_surface_check, run_boundary_profile_render_matrix_check,
};

fn collect_rendered_env_keys(rendered_yaml: &str) -> std::collections::BTreeSet<String> {
    fn collect_from_value(
        value: &serde_yaml::Value,
        env_keys: &mut std::collections::BTreeSet<String>,
    ) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, child) in map {
                    if let Some(key_text) = key.as_str() {
                        if (key_text.starts_with("ATLAS_") || key_text.starts_with("BIJUX_"))
                            && key_text.len() > "ATLAS_".len()
                        {
                            env_keys.insert(key_text.to_string());
                        }
                        if key_text == "name" {
                            if let Some(env_name) = child.as_str() {
                                if (env_name.starts_with("ATLAS_")
                                    || env_name.starts_with("BIJUX_"))
                                    && env_name.len() > "ATLAS_".len()
                                {
                                    env_keys.insert(env_name.to_string());
                                }
                            }
                        }
                    }
                    collect_from_value(child, env_keys);
                }
            }
            serde_yaml::Value::Sequence(items) => {
                for child in items {
                    collect_from_value(child, env_keys);
                }
            }
            _ => {}
        }
    }

    let mut env_keys = std::collections::BTreeSet::<String>::new();
    for document in serde_yaml::Deserializer::from_str(rendered_yaml) {
        let value = match serde_yaml::Value::deserialize(document) {
            Ok(value) => value,
            Err(_) => continue,
        };
        collect_from_value(&value, &mut env_keys);
    }
    env_keys
}

fn violation(
    contract_id: &str,
    test_id: &str,
    file: Option<String>,
    message: impl Into<String>,
) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: None,
        message: message.into(),
        evidence: None,
    }
}

fn write_boundary_report(
    ctx: &RunContext,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let out_dir = ctx
        .artifacts_root
        .clone()
        .unwrap_or_else(|| ctx.repo_root.join("artifacts/contracts/repo"))
        .join("boundary-closure");
    fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
    let path = out_dir.join(file_name);
    fs::write(
        &path,
        serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    Ok(path
        .strip_prefix(&ctx.repo_root)
        .unwrap_or(path.as_path())
        .display()
        .to_string())
}

fn read_env_allowlist(root: &Path) -> Result<std::collections::BTreeSet<String>, String> {
    let schema_path = root.join("configs/contracts/env.schema.json");
    let schema_text = fs::read_to_string(&schema_path)
        .map_err(|err| format!("read {} failed: {err}", schema_path.display()))?;
    let schema_json: serde_json::Value =
        serde_json::from_str(&schema_text).map_err(|err| format!("invalid json: {err}"))?;
    Ok(schema_json["allowed_env"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str())
        .map(str::to_string)
        .collect())
}

fn verify_declared_ops_tools(root: &Path, required_tools: &[&str]) -> Result<Vec<String>, String> {
    let toolchain_path = root.join("ops/inventory/toolchain.json");
    let toolchain_text = fs::read_to_string(&toolchain_path)
        .map_err(|err| format!("read {} failed: {err}", toolchain_path.display()))?;
    let toolchain_json: serde_json::Value =
        serde_json::from_str(&toolchain_text).map_err(|err| format!("invalid json: {err}"))?;
    let tools = toolchain_json
        .get("tools")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "ops toolchain inventory must declare a tools object".to_string())?;
    let mut missing = Vec::new();
    for tool in required_tools {
        if !tools.contains_key(*tool) {
            missing.push((*tool).to_string());
        }
    }
    Ok(missing)
}

fn count_files(root: &Path) -> usize {
    let mut total = 0usize;
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += count_files(&path);
            } else if path.is_file() {
                total += 1;
            }
        }
    }
    total
}

fn parse_mkdocs_site_dir(root: &Path) -> Result<String, String> {
    crate::docs::site_output::parse_mkdocs_site_paths(root)
        .map(|paths| paths.site_dir.display().to_string())
}

fn run_sanitized_output(
    root: &Path,
    binary: &str,
    args: &[&str],
) -> Result<std::process::Output, String> {
    let mut command = Command::new(binary);
    command.current_dir(root).args(args).env_clear();
    for key in [
        "HOME", "PATH", "TMPDIR", "TEMP", "TMP", "USER", "LOGNAME", "SHELL",
    ] {
        if let Ok(value) = std::env::var(key) {
            command.env(key, value);
        }
    }
    command
        .output()
        .map_err(|err| format!("failed to start `{binary}`: {err}"))
}

fn run_boundary_docs_output_dir_check(ctx: &RunContext) -> (serde_json::Value, Vec<Violation>) {
    let contract_id = "REPO-005";
    let test_id = "repo.docs_output_dir.preview_contains_site_payload";
    let mut violations = Vec::new();
    let site_dir = match parse_mkdocs_site_dir(&ctx.repo_root) {
        Ok(site_dir) => site_dir,
        Err(err) => {
            violations.push(violation(
                contract_id,
                test_id,
                Some("mkdocs.yml".to_string()),
                err,
            ));
            "artifacts/docs/site".to_string()
        }
    };

    let build = Command::new("mkdocs")
        .current_dir(&ctx.repo_root)
        .args(["build", "--strict"])
        .output();
    match build {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            violations.push(violation(
                contract_id,
                test_id,
                Some("mkdocs.yml".to_string()),
                if stderr.is_empty() {
                    "mkdocs build --strict must succeed".to_string()
                } else {
                    format!("mkdocs build --strict must succeed: {stderr}")
                },
            ));
        }
        Err(err) => {
            violations.push(violation(
                contract_id,
                test_id,
                Some("mkdocs.yml".to_string()),
                format!("failed to run mkdocs build --strict: {err}"),
            ));
        }
    }

    let site_root = ctx.repo_root.join(&site_dir);
    let site_output = match crate::docs::site_output::collect_site_output_status(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            violations.push(violation(
                contract_id,
                test_id,
                Some("configs/docs/site-output-contract.json".to_string()),
                err,
            ));
            crate::docs::site_output::SiteOutputStatus {
                docs_dir: std::path::PathBuf::from("docs"),
                site_dir: std::path::PathBuf::from(&site_dir),
                site_dir_exists: site_root.is_dir(),
                index_exists: site_root.join("index.html").is_file(),
                assets_exists: site_root.join("assets").is_dir(),
                file_count: count_files(&site_root),
                minimum_file_count: 10,
                assets_directory: "assets".to_string(),
            }
        }
    };
    let index_exists = site_output.index_exists;
    let assets_exists = site_output.assets_exists;
    let internal_dir_exists = site_root.join("_internal").exists();
    let redirect_output_exists = site_root
        .join("root/architecture-overview/index.html")
        .is_file();
    let file_count = site_output.file_count;
    if !index_exists {
        violations.push(violation(
            contract_id,
            test_id,
            Some(format!("{site_dir}/index.html")),
            "docs build output must include index.html",
        ));
    }
    if !assets_exists {
        violations.push(violation(
            contract_id,
            test_id,
            Some(format!("{site_dir}/assets")),
            "docs build output must include assets directory",
        ));
    }
    if file_count < site_output.minimum_file_count {
        violations.push(violation(
            contract_id,
            test_id,
            Some(site_dir.clone()),
            "docs build output must contain a non-trivial file count",
        ));
    }
    if !redirect_output_exists {
        violations.push(violation(
            contract_id,
            test_id,
            Some(format!("{site_dir}/root/architecture-overview/index.html")),
            "docs build output must keep the known legacy redirect target materialized",
        ));
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "check": "docs_output_dir",
        "status": if violations.is_empty() { "pass" } else { "fail" },
        "site_dir": site_dir,
        "checks": [
            {"id": "DOCS-SITE-001", "pass": site_output.site_dir_exists},
            {"id": "DOCS-SITE-002", "pass": index_exists},
            {"id": "DOCS-SITE-003", "pass": assets_exists},
        ],
        "index_exists": index_exists,
        "assets_exists": assets_exists,
        "internal_dir_exists": internal_dir_exists,
        "redirect_output_exists": redirect_output_exists,
        "file_count": file_count,
        "minimum_file_count": site_output.minimum_file_count,
    });
    if let Err(err) = write_boundary_report(ctx, "docs-output-dir.json", &payload) {
        violations.push(violation(
            contract_id,
            test_id,
            Some("artifacts/contracts/repo/boundary-closure/docs-output-dir.json".to_string()),
            err,
        ));
    }
    (payload, violations)
}

fn run_boundary_closure_summary(ctx: &RunContext) -> (serde_json::Value, Vec<Violation>) {
    let (env_payload, mut violations) = run_boundary_helm_env_surface_check(ctx);
    let (profiles_payload, mut profile_violations) = run_boundary_profile_render_matrix_check(ctx);
    let (docs_payload, mut docs_violations) = run_boundary_docs_output_dir_check(ctx);
    violations.append(&mut profile_violations);
    violations.append(&mut docs_violations);

    let summary = serde_json::json!({
        "schema_version": 1,
        "kind": "boundary_closure_summary",
        "status": if violations.is_empty() { "pass" } else { "fail" },
        "checks": [
            {"name": "helm_env_surface", "status": env_payload["status"]},
            {"name": "k8s_profile_render_matrix", "status": profiles_payload["status"]},
            {"name": "docs_output_dir", "status": docs_payload["status"]},
        ],
        "why_this_exists": "These closure checks prevent the Helm env drift, install-profile drift, and docs output drift found in the audits."
    });
    if let Err(err) = write_boundary_report(ctx, "summary.json", &summary) {
        violations.push(violation(
            "REPO-006",
            "repo.boundary_closure.summary_report_generated",
            Some("artifacts/contracts/repo/boundary-closure/summary.json".to_string()),
            err,
        ));
    }
    (summary, violations)
}

fn test_repo_001_law_registry_exists_and_is_valid(ctx: &RunContext) -> TestResult {
    let rel = "docs/_internal/contracts/repo-laws.json";
    let path = ctx.repo_root.join(rel);
    let text = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-001",
                "repo.laws.registry_present",
                Some(rel.to_string()),
                format!("read failed: {err}"),
            )])
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "REPO-001",
                "repo.laws.registry_present",
                Some(rel.to_string()),
                format!("invalid json: {err}"),
            )])
        }
    };
    let mut violations = Vec::new();
    if json.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        violations.push(violation(
            "REPO-001",
            "repo.laws.registry_present",
            Some(rel.to_string()),
            "repo laws registry must declare schema_version=1",
        ));
    }
    if json.get("laws").and_then(|v| v.as_array()).is_none() {
        violations.push(violation(
            "REPO-001",
            "repo.laws.registry_present",
            Some(rel.to_string()),
            "repo laws registry must contain a laws array",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_repo_002_root_allowlist_config_present(ctx: &RunContext) -> TestResult {
    let rel = "configs/repo/root-file-allowlist.json";
    if ctx.repo_root.join(rel).exists() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "REPO-002",
            "repo.surface.root_allowlist_present",
            Some(rel.to_string()),
            "root allowlist config is missing",
        )])
    }
}

fn test_repo_003_helm_env_surface_subset_of_runtime_contract(ctx: &RunContext) -> TestResult {
    if !ctx.allow_subprocess {
        return TestResult::Skip("helm env surface check requires --allow-subprocess".to_string());
    }
    let (_, mut violations) = run_boundary_helm_env_surface_check(ctx);
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_repo_004_k8s_profile_render_matrix_is_installable_by_construction(
    ctx: &RunContext,
) -> TestResult {
    if !ctx.allow_subprocess {
        return TestResult::Skip(
            "k8s profile render matrix requires --allow-subprocess".to_string(),
        );
    }
    let (_, mut violations) = run_boundary_profile_render_matrix_check(ctx);
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_repo_005_docs_output_dir_contains_preview_payload(ctx: &RunContext) -> TestResult {
    if !ctx.allow_subprocess {
        return TestResult::Skip("docs output dir check requires --allow-subprocess".to_string());
    }
    let (_, mut violations) = run_boundary_docs_output_dir_check(ctx);
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

fn test_repo_006_boundary_closure_summary_is_generated(ctx: &RunContext) -> TestResult {
    if !ctx.allow_subprocess {
        return TestResult::Skip(
            "boundary closure summary requires --allow-subprocess".to_string(),
        );
    }
    let (_, mut violations) = run_boundary_closure_summary(ctx);
    if violations.is_empty() {
        TestResult::Pass
    } else {
        violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
        TestResult::Fail(violations)
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("REPO-001".to_string()),
            title: "repo laws registry remains valid and parseable",
            tests: vec![TestCase {
                id: TestId("repo.laws.registry_present".to_string()),
                title: "repo laws registry exists and parses",
                kind: TestKind::Pure,
                run: test_repo_001_law_registry_exists_and_is_valid,
            }],
        },
        Contract {
            id: ContractId("REPO-002".to_string()),
            title: "repo root allowlist config remains present",
            tests: vec![TestCase {
                id: TestId("repo.surface.root_allowlist_present".to_string()),
                title: "root allowlist config exists",
                kind: TestKind::Pure,
                run: test_repo_002_root_allowlist_config_present,
            }],
        },
        Contract {
            id: ContractId("REPO-003".to_string()),
            title: "helm-emitted env keys stay inside the runtime allowlist",
            tests: vec![TestCase {
                id: TestId("repo.helm_env_surface.subset_of_runtime_allowlist".to_string()),
                title: "helm-emitted env keys are a subset of configs/contracts/env.schema.json",
                kind: TestKind::Subprocess,
                run: test_repo_003_helm_env_surface_subset_of_runtime_contract,
            }],
        },
        Contract {
            id: ContractId("REPO-004".to_string()),
            title: "k8s install profiles render cleanly by construction",
            tests: vec![TestCase {
                id: TestId(
                    "repo.k8s_profile_render_matrix.installable_by_construction".to_string(),
                ),
                title: "install-matrix profiles pass helm lint, helm template, and kubeconform",
                kind: TestKind::Subprocess,
                run: test_repo_004_k8s_profile_render_matrix_is_installable_by_construction,
            }],
        },
        Contract {
            id: ContractId("REPO-005".to_string()),
            title: "docs build output stays aligned with mkdocs site_dir",
            tests: vec![TestCase {
                id: TestId("repo.docs_output_dir.preview_contains_site_payload".to_string()),
                title: "mkdocs site_dir build output contains index and assets",
                kind: TestKind::Subprocess,
                run: test_repo_005_docs_output_dir_contains_preview_payload,
            }],
        },
        Contract {
            id: ContractId("REPO-006".to_string()),
            title: "boundary closure summary is generated from effect checks",
            tests: vec![TestCase {
                id: TestId("repo.boundary_closure.summary_report_generated".to_string()),
                title:
                    "boundary closure summary report records pass or fail for each closure check",
                kind: TestKind::Subprocess,
                run: test_repo_006_boundary_closure_summary_is_generated,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "REPO-001" => {
            "Ensures canonical repo law registry exists and is valid JSON with required metadata."
                .to_string()
        }
        "REPO-002" => "Ensures root allowlist authority config exists for root surface governance."
            .to_string(),
        "REPO-003" => "Ensures the rendered Helm env surface cannot drift outside the runtime env allowlist contract."
            .to_string(),
        "REPO-004" => "Ensures every install-matrix profile remains renderable, schema-valid, and kubeconform-clean without cluster access."
            .to_string(),
        "REPO-005" => "Ensures mkdocs strict builds to the configured site_dir and the preview payload contains real site content."
            .to_string(),
        "REPO-006" => "Ensures the boundary closure summary report records the three audit-driven closure checks and their pass or fail state."
            .to_string(),
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts repo`.".to_string(),
    }
}

pub fn contract_gate_command(contract_id: &str) -> &'static str {
    match contract_id {
        "REPO-003" | "REPO-004" | "REPO-005" | "REPO-006" => {
            "bijux dev atlas contracts repo --mode effect --allow-subprocess"
        }
        _ => "bijux dev atlas contracts repo --mode static",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crate_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn env_surface_fixture_keeps_the_allowlist_mismatch_detectable() {
        let root = crate_root();
        let rendered = fs::read_to_string(
            root.join("tests/fixtures/repo_boundary_closure/env-surface-mismatch.yaml"),
        )
        .expect("read env surface fixture");
        let allowlist_text = fs::read_to_string(
            root.join("tests/fixtures/repo_boundary_closure/env-allowlist.json"),
        )
        .expect("read env allowlist fixture");
        let allowlist_json: serde_json::Value =
            serde_json::from_str(&allowlist_text).expect("parse env allowlist fixture");
        let allowed = allowlist_json["allowed_env"]
            .as_array()
            .expect("allowed_env array")
            .iter()
            .filter_map(|value| value.as_str())
            .map(str::to_string)
            .collect::<std::collections::BTreeSet<_>>();

        let emitted = collect_rendered_env_keys(&rendered);
        let missing = emitted.difference(&allowed).cloned().collect::<Vec<_>>();
        assert_eq!(
            missing,
            vec!["ATLAS_UNDECLARED".to_string()],
            "fixture must keep the undeclared env mismatch visible"
        );
    }

    #[test]
    fn install_matrix_fixture_keeps_the_missing_values_file_detectable() {
        let root = crate_root();
        let matrix_text = fs::read_to_string(
            root.join("tests/fixtures/repo_boundary_closure/install-matrix-mismatch.json"),
        )
        .expect("read install matrix fixture");
        let matrix_json: serde_json::Value =
            serde_json::from_str(&matrix_text).expect("parse install matrix fixture");
        let profiles = matrix_json["profiles"].as_array().expect("profiles array");
        let missing = profiles
            .iter()
            .filter_map(|profile| profile.get("values_file").and_then(|value| value.as_str()))
            .filter(|values_file| !root.join(values_file).is_file())
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            missing,
            vec!["tests/fixtures/repo_boundary_closure/missing-values.yaml".to_string()],
            "fixture must keep the missing profile values reference visible"
        );
    }
}
