// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    TutorialsBuildCommand, TutorialsBuildDocsArgs, TutorialsCommand, TutorialsCommandArgs,
    TutorialsContractsCommand, TutorialsDashboardsCommand, TutorialsDatasetCommand,
    TutorialsEvidenceCommand, TutorialsRunCommand, TutorialsWorkspaceCommand,
    TutorialsWorkspaceCleanupArgs,
};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::domains::tutorials::checks;
use bijux_dev_atlas::domains::tutorials::contracts;
use bijux_dev_atlas::domains::tutorials::runtime::workspace::TutorialWorkspaceManager;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

pub(crate) fn run_tutorials_command(quiet: bool, command: TutorialsCommand) -> i32 {
    let run = match command {
        TutorialsCommand::List(args) => run_tutorials_list(&args),
        TutorialsCommand::Explain(args) => run_tutorials_explain(&args),
        TutorialsCommand::Verify(args) => run_tutorials_verify(&args),
        TutorialsCommand::Run { command } => match command {
            TutorialsRunCommand::Workflow(args) => run_tutorials_workflow(&args),
        },
        TutorialsCommand::Build { command } => match command {
            TutorialsBuildCommand::Docs(args) => run_tutorials_build_docs(&args),
        },
        TutorialsCommand::Dataset { command } => match command {
            TutorialsDatasetCommand::Package(args) => run_tutorials_dataset_package(&args),
            TutorialsDatasetCommand::Ingest(args) => run_tutorials_dataset_ingest(&args),
            TutorialsDatasetCommand::IntegrityCheck(args) => {
                run_tutorials_dataset_integrity_check(&args)
            }
        },
        TutorialsCommand::ReproducibilityCheck(args) => run_tutorials_reproducibility_check(&args),
        TutorialsCommand::Workspace { command } => match command {
            TutorialsWorkspaceCommand::Cleanup(args) => run_tutorials_workspace_cleanup(&args),
        },
        TutorialsCommand::Dashboards { command } => match command {
            TutorialsDashboardsCommand::Validate(args) => run_tutorials_dashboards_validate(&args),
        },
        TutorialsCommand::Evidence { command } => match command {
            TutorialsEvidenceCommand::Validate(args) => run_tutorials_evidence_validate(&args),
        },
        TutorialsCommand::Contracts { command } => match command {
            TutorialsContractsCommand::Validate(args) => run_tutorials_contracts_validate(&args),
        },
        TutorialsCommand::Generate(args) => run_tutorials_generate(&args),
    };

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas tutorials failed: {err}");
            1
        }
    }
}

#[derive(Debug, Clone)]
struct TutorialAssets {
    dataset_contract: serde_json::Value,
    evidence_items: Vec<(String, serde_json::Value)>,
    dashboard_items: Vec<(String, serde_json::Value)>,
    contract_files: Vec<String>,
}

fn run_tutorials_list(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "list",
        "repo_root": repo_root.display().to_string(),
        "counts": {
            "contract_files": assets.contract_files.len(),
            "evidence_items": assets.evidence_items.len(),
            "dashboard_items": assets.dashboard_items.len()
        },
        "paths": {
            "dataset_contract": "tutorials/contracts/tutorial-dataset-contract.json",
            "contracts_directory": "tutorials/contracts",
            "evidence_directory": "tutorials/evidence",
            "dashboards_directory": "tutorials/dashboards"
        },
        "files": {
            "contracts": assets.contract_files,
            "evidence": assets.evidence_items.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
            "dashboards": assets.dashboard_items.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>()
        }
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_tutorials_explain(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let generated_outputs = vec![
        "artifacts/tutorials/inventory-health.json",
        "artifacts/tutorials/verify-report.json",
        "artifacts/tutorials/workflow-report.json",
    ];
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "explain",
        "repo_root": repo_root.display().to_string(),
        "doctrine": {
            "automation_owner": "bijux-dev-atlas",
            "artifacts_only_directory": "tutorials/"
        },
        "source_assets": {
            "dataset_contract": "tutorials/contracts/tutorial-dataset-contract.json",
            "evidence_items": assets.evidence_items.len(),
            "dashboard_items": assets.dashboard_items.len()
        },
        "generated_outputs": generated_outputs,
        "command_surface": [
            "tutorials list",
            "tutorials explain",
            "tutorials verify",
            "tutorials run workflow",
            "tutorials build docs",
            "tutorials dataset package",
            "tutorials dataset ingest",
            "tutorials dataset integrity-check",
            "tutorials reproducibility-check",
            "tutorials workspace cleanup",
            "tutorials dashboards validate",
            "tutorials evidence validate",
            "tutorials contracts validate",
            "tutorials generate"
        ]
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_tutorials_verify(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let tutorial_contracts = contracts::contracts(&repo_root)?;
    let tutorial_checks = checks::checks(&repo_root)?;
    let (dashboards_ok, dashboards_detail) = validate_dashboards(&assets.dashboard_items);
    let (evidence_ok, evidence_detail) = validate_evidence(&assets.evidence_items);
    let (contract_ok, contract_detail) = validate_dataset_contract_semantics(&assets.dataset_contract);
    let success = dashboards_ok && evidence_ok && contract_ok;
    let report = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "verify",
        "success": success,
        "checks": {
            "dashboards": dashboards_detail,
            "evidence": evidence_detail,
            "dataset_contract": contract_detail
        },
        "catalog": {
            "domain_contract_entries": tutorial_contracts.len(),
            "domain_check_entries": tutorial_checks.len()
        }
    });
    write_tutorial_report(&repo_root, "verify-report.json", &report)?;
    let rendered = emit_payload(args.format, args.out.clone(), &report)?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_workflow(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let steps = vec![
        step_result("contracts-validate", run_tutorials_contracts_validate(args).map(|_| ())),
        step_result("dashboards-validate", run_tutorials_dashboards_validate(args).map(|_| ())),
        step_result("evidence-validate", run_tutorials_evidence_validate(args).map(|_| ())),
        step_result("verify", run_tutorials_verify(args).map(|_| ())),
    ];
    let failures = steps
        .iter()
        .filter(|row| !row["success"].as_bool().unwrap_or(false))
        .count();
    let report = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "run-workflow",
        "success": failures == 0,
        "steps": steps,
        "summary": {
            "total": 4,
            "passed": 4 - failures,
            "failed": failures
        }
    });
    write_tutorial_report(&repo_root, "workflow-report.json", &report)?;
    let rendered = emit_payload(args.format, args.out.clone(), &report)?;
    Ok((rendered, if failures == 0 { 0 } else { 1 }))
}

fn run_tutorials_build_docs(args: &TutorialsBuildDocsArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let mut cmd = ProcessCommand::new("mkdocs");
    cmd.current_dir(&repo_root).arg("build");
    if args.strict {
        cmd.arg("--strict");
    }
    if let Some(site_dir) = &args.site_dir {
        cmd.arg("--site-dir").arg(site_dir);
    }
    let status = cmd
        .status()
        .map_err(|err| format!("failed to execute mkdocs build: {err}"))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "build-docs",
        "strict": args.strict,
        "site_dir": args.site_dir.as_ref().map(|p| p.display().to_string()),
        "success": status.success()
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, if status.success() { 0 } else { 1 }))
}

fn run_tutorials_dataset_package(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let files = collect_dataset_files(&assets.dataset_contract);
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "dataset-package",
        "text": "dataset packager command surface registered",
        "contract_files": files
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_tutorials_dataset_ingest(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "dataset-ingest",
        "text": "tutorial dataset ingest command surface registered",
        "repo_root": repo_root.display().to_string()
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_tutorials_dataset_integrity_check(
    args: &TutorialsCommandArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let files = collect_dataset_files(&assets.dataset_contract);
    let mut missing = Vec::new();
    for file in files {
        let candidate = repo_root.join(file.as_str());
        if !candidate.exists() {
            missing.push(file);
        }
    }
    let success = missing.is_empty();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "dataset-integrity-check",
        "success": success,
        "missing_files": missing
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_reproducibility_check(
    args: &TutorialsCommandArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let files = collect_dataset_files(&assets.dataset_contract);
    let mut signatures = BTreeMap::new();
    for file in files {
        let candidate = repo_root.join(file.as_str());
        if candidate.exists() {
            let bytes = fs::read(&candidate)
                .map_err(|err| format!("failed to read {}: {err}", candidate.display()))?;
            let digest = sha256_hex(&bytes);
            signatures.insert(file, digest);
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "reproducibility-check",
        "text": "baseline dataset signatures collected for reproducibility comparison",
        "signatures": signatures
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_tutorials_workspace_cleanup(
    args: &TutorialsWorkspaceCleanupArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let workspace = TutorialWorkspaceManager::new(&repo_root);
    workspace.ensure()?;
    let removed = workspace.safe_cleanup(workspace.root(), args.dry_run)?;
    if !args.dry_run {
        workspace.ensure()?;
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "workspace-cleanup",
        "workspace_root": workspace.root().display().to_string(),
        "dry_run": args.dry_run,
        "removed": removed
    });
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

fn run_tutorials_dashboards_validate(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let (success, detail) = validate_dashboards(&assets.dashboard_items);
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "dashboards-validate",
        "success": success,
        "detail": detail
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_evidence_validate(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let (success, detail) = validate_evidence(&assets.evidence_items);
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "evidence-validate",
        "success": success,
        "detail": detail
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_contracts_validate(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let (success, detail) = validate_dataset_contract_semantics(&assets.dataset_contract);
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "contracts-validate",
        "success": success,
        "detail": detail
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_generate(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let report = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "generate",
        "text": "tutorial derived artifacts regenerated deterministically",
        "inventory": {
            "dashboards": assets.dashboard_items.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
            "evidence": assets.evidence_items.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
            "contracts": assets.contract_files
        }
    });
    write_tutorial_report(&repo_root, "inventory-health.json", &report)?;
    let rendered = emit_payload(args.format, args.out.clone(), &report)?;
    Ok((rendered, 0))
}

fn load_tutorial_assets(repo_root: &Path) -> Result<TutorialAssets, String> {
    let dataset_contract_path = repo_root.join("tutorials/contracts/tutorial-dataset-contract.json");
    let dataset_contract_text = fs::read_to_string(&dataset_contract_path)
        .map_err(|err| format!("failed to read {}: {err}", dataset_contract_path.display()))?;
    let dataset_contract: serde_json::Value = serde_json::from_str(&dataset_contract_text)
        .map_err(|err| format!("failed to parse {}: {err}", dataset_contract_path.display()))?;

    let evidence_items = load_json_directory(repo_root.join("tutorials/evidence"))?;
    let dashboard_items = load_json_directory(repo_root.join("tutorials/dashboards"))?;
    let contract_files = scan_contracts_directory(repo_root.join("tutorials/contracts"))?;

    Ok(TutorialAssets {
        dataset_contract,
        evidence_items,
        dashboard_items,
        contract_files,
    })
}

fn scan_contracts_directory(path: PathBuf) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))? {
        let entry = entry.map_err(|err| format!("failed to read entry: {err}"))?;
        let p = entry.path();
        if p.is_file() {
            files.push(
                p.file_name()
                    .and_then(|v| v.to_str())
                    .ok_or_else(|| format!("invalid utf-8 filename under {}", path.display()))?
                    .to_string(),
            );
        }
    }
    files.sort();
    let expected = ["tutorial-dataset-contract.json", "atlas-example-minimal.sha256"];
    let missing = expected
        .iter()
        .filter(|item| !files.iter().any(|found| found == **item))
        .map(|item| item.to_string())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "tutorial contracts directory is missing expected files: {}",
            missing.join(", ")
        ));
    }
    Ok(files)
}

fn load_json_directory(path: PathBuf) -> Result<Vec<(String, serde_json::Value)>, String> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))? {
        let entry = entry.map_err(|err| format!("failed to read entry: {err}"))?;
        let p = entry.path();
        if p.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&p).map_err(|err| format!("failed to read {}: {err}", p.display()))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", p.display()))?;
        let name = p
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or_else(|| format!("invalid utf-8 filename under {}", path.display()))?
            .to_string();
        rows.push((name, json));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(rows)
}

fn validate_dashboards(
    dashboards: &[(String, serde_json::Value)],
) -> (bool, serde_json::Value) {
    let mut violations = Vec::new();
    for (name, dashboard) in dashboards {
        let has_title = dashboard.get("title").and_then(|v| v.as_str()).is_some();
        let has_panels = dashboard.get("panels").and_then(|v| v.as_array()).is_some();
        if !has_title || !has_panels {
            violations.push(format!("{name}: missing required `title` or `panels`"));
        }
    }
    (
        violations.is_empty(),
        serde_json::json!({
            "checked": dashboards.len(),
            "violations": violations
        }),
    )
}

fn validate_evidence(
    evidence: &[(String, serde_json::Value)],
) -> (bool, serde_json::Value) {
    let mut violations = Vec::new();
    for (name, item) in evidence {
        let has_id = item.get("id").is_some();
        let has_summary = item.get("summary").is_some() || item.get("metrics").is_some();
        if !has_id || !has_summary {
            violations.push(format!("{name}: missing required evidence keys"));
        }
    }
    (
        violations.is_empty(),
        serde_json::json!({
            "checked": evidence.len(),
            "violations": violations
        }),
    )
}

fn validate_dataset_contract_semantics(contract: &serde_json::Value) -> (bool, serde_json::Value) {
    let files = collect_dataset_files(contract);
    let mut violations = Vec::new();
    if files.is_empty() {
        violations.push("dataset contract must declare at least one file".to_string());
    }
    if contract.get("dataset").is_none() {
        violations.push("dataset contract must declare `dataset` metadata".to_string());
    }
    (
        violations.is_empty(),
        serde_json::json!({
            "declared_files": files,
            "violations": violations
        }),
    )
}

fn collect_dataset_files(contract: &serde_json::Value) -> Vec<String> {
    let mut files = Vec::new();
    if let Some(array) = contract.get("files").and_then(|v| v.as_array()) {
        for entry in array {
            if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                files.push(path.to_string());
            } else if let Some(path) = entry.as_str() {
                files.push(path.to_string());
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

fn write_tutorial_report(repo_root: &Path, file_name: &str, payload: &serde_json::Value) -> Result<(), String> {
    let path = repo_root.join("artifacts/tutorials").join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(payload).map_err(|err| format!("serialize report failed: {err}"))?;
    fs::write(&path, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn step_result(name: &str, result: Result<(), String>) -> serde_json::Value {
    match result {
        Ok(()) => serde_json::json!({
            "name": name,
            "success": true
        }),
        Err(err) => serde_json::json!({
            "name": name,
            "success": false,
            "error": err
        }),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
