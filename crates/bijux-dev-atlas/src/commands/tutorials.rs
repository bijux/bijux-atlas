// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    TutorialsBuildCommand, TutorialsBuildDocsArgs, TutorialsCommand, TutorialsCommandArgs,
    TutorialsContractsCommand, TutorialsDashboardsCommand, TutorialsDatasetCommand,
    TutorialsDatasetPackageArgs, TutorialsEvidenceCommand, TutorialsRealDataCommand,
    TutorialsRealDataPlanArgs, TutorialsRealDataRunArgs, TutorialsRunCommand, TutorialsWorkflowArgs,
    TutorialsWorkspaceCleanupArgs, TutorialsWorkspaceCommand,
};
use crate::{emit_payload, resolve_repo_root};
use bijux_dev_atlas::domains::tutorials::checks;
use bijux_dev_atlas::domains::tutorials::contracts;
use bijux_dev_atlas::domains::tutorials::runtime::workspace::TutorialWorkspaceManager;
use bijux_dev_atlas::ui::terminal::report::{render_status_line, LineStyle};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

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
            TutorialsContractsCommand::Explain(args) => run_tutorials_contracts_explain(&args),
        },
        TutorialsCommand::RealData { command } => match command {
            TutorialsRealDataCommand::List(args) => run_tutorials_real_data_list(&args),
            TutorialsRealDataCommand::Plan(args) => run_tutorials_real_data_plan(&args),
            TutorialsRealDataCommand::Fetch(args) => run_tutorials_real_data_fetch(&args),
            TutorialsRealDataCommand::Ingest(args) => run_tutorials_real_data_ingest(&args),
            TutorialsRealDataCommand::Doctor(args) => run_tutorials_real_data_doctor(&args),
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
    let rendered = emit_tutorial_output(args, &payload, None)?;
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
            "tutorials contracts explain",
            "tutorials generate"
        ]
    });
    let rendered = emit_tutorial_output(args, &payload, Some(render_tutorial_explain_markdown()))?;
    Ok((rendered, 0))
}

fn run_tutorials_verify(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let tutorial_contracts = contracts::contracts(&repo_root)?;
    let tutorial_checks = checks::checks(&repo_root)?;
    let (dashboards_ok, dashboards_detail) = validate_dashboards(&assets.dashboard_items);
    let (evidence_ok, evidence_detail) = validate_evidence(&assets.evidence_items);
    let (contract_ok, contract_detail) =
        validate_dataset_contract_semantics(&assets.dataset_contract);
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
    write_tutorial_markdown_report(
        &repo_root,
        "verify-report.md",
        &render_tutorial_summary_markdown("verify", &report),
    )?;
    let rendered = emit_tutorial_output(
        args,
        &report,
        Some(render_tutorial_summary_markdown("verify", &report)),
    )?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_workflow(args: &TutorialsWorkflowArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let mut selected = vec![
        "setup",
        "ingest",
        "query",
        "dashboards-validate",
        "evidence-validate",
    ];
    if let Some(only) = args.only.as_deref() {
        selected = vec![only];
    }
    let mut steps = Vec::new();
    for step in selected {
        let row = match step {
            "setup" => run_timed_step("setup", || run_tutorials_workspace_setup(&args.common)),
            "ingest" => run_timed_step("ingest", || {
                run_tutorials_dataset_ingest(&args.common).map(|_| ())
            }),
            "query" => run_timed_step("query", || run_tutorials_query_probe(&args.common)),
            "dashboards-validate" => run_timed_step("dashboards-validate", || {
                run_tutorials_dashboards_validate(&args.common).map(|_| ())
            }),
            "evidence-validate" => run_timed_step("evidence-validate", || {
                run_tutorials_evidence_validate(&args.common).map(|_| ())
            }),
            "verify" => run_timed_step("verify", || run_tutorials_verify(&args.common).map(|_| ())),
            unknown => serde_json::json!({
                "name": unknown,
                "success": false,
                "error": format!("unknown workflow step `{unknown}`"),
                "duration_ms": 0
            }),
        };
        steps.push(row);
    }
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
            "total": steps.len(),
            "passed": steps.len().saturating_sub(failures),
            "failed": failures
        }
    });
    write_tutorial_report(&repo_root, "workflow-report.json", &report)?;
    write_tutorial_markdown_report(
        &repo_root,
        "workflow-report.md",
        &render_tutorial_summary_markdown("run-workflow", &report),
    )?;
    let rendered = match args.common.format {
        crate::cli::FormatArg::Text if !args.common.markdown => {
            render_workflow_nextest(&steps, !args.common.no_color, args.common.verbose)
        }
        _ => emit_tutorial_output(
            &args.common,
            &report,
            Some(render_tutorial_summary_markdown("run-workflow", &report)),
        )?,
    };
    let rendered = if args.common.quiet {
        String::new()
    } else {
        rendered
    };
    Ok((rendered, if failures == 0 { 0 } else { 1 }))
}

fn run_tutorials_build_docs(args: &TutorialsBuildDocsArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let mut cmd = ProcessCommand::new("mkdocs");
    cmd.current_dir(&repo_root).arg("build");
    cmd.env("SOURCE_DATE_EPOCH", "0")
        .env("PYTHONHASHSEED", "0")
        .env("LC_ALL", "C")
        .env("TZ", "UTC");
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
    let rendered = emit_tutorial_output(&args.common, &payload, None)?;
    Ok((rendered, if status.success() { 0 } else { 1 }))
}

fn run_tutorials_dataset_package(
    args: &TutorialsDatasetPackageArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let sha_file_path = repo_root.join("tutorials/contracts/atlas-example-minimal.sha256");
    let files = load_files_from_sha256_manifest(&sha_file_path)?;
    let package_dir = repo_root.join("artifacts/tutorials/datasets");
    fs::create_dir_all(&package_dir)
        .map_err(|err| format!("failed to create {}: {err}", package_dir.display()))?;
    let package_path = package_dir.join("atlas-example-minimal.tar");
    build_deterministic_tar(&repo_root, &files, &package_path, args.stable_timestamps)?;
    if args.update_sha256 {
        update_sha256_manifest(&repo_root, &files, &sha_file_path)?;
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "dataset-package",
        "text": "dataset packaged with deterministic ordering",
        "input_files": files,
        "package_path": package_path.display().to_string(),
        "stable_timestamps": args.stable_timestamps,
        "sha256_manifest_updated": args.update_sha256
    });
    let rendered = emit_tutorial_output(&args.common, &payload, None)?;
    Ok((rendered, 0))
}

fn run_tutorials_dataset_ingest(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let started = Instant::now();
    let output = ProcessCommand::new("cargo")
        .current_dir(&repo_root)
        .args([
            "run",
            "-q",
            "-p",
            "bijux-dev-atlas",
            "--",
            "ingest",
            "dry-run",
            "--format",
            "json",
        ])
        .output()
        .map_err(|err| format!("failed to run ingest dry-run: {err}"))?;
    let stdout =
        String::from_utf8(output.stdout).map_err(|err| format!("ingest stdout utf8: {err}"))?;
    let stderr =
        String::from_utf8(output.stderr).map_err(|err| format!("ingest stderr utf8: {err}"))?;
    let parsed = serde_json::from_str::<serde_json::Value>(&stdout).unwrap_or_else(|_| {
        serde_json::json!({
            "raw_stdout": stdout
        })
    });
    let evidence = serde_json::json!({
        "schema_version": 1,
        "kind": "tutorial_ingest_evidence",
        "status": if output.status.success() { "pass" } else { "fail" },
        "exit_code": output.status.code(),
        "duration_ms": started.elapsed().as_millis() as u64,
        "ingest_output": parsed
    });
    write_tutorial_report(&repo_root, "ingest-evidence.json", &evidence)?;
    write_tutorial_log(&repo_root, "ingest.log", &stdout, &stderr)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "dataset-ingest",
        "text": "tutorial dataset ingest completed",
        "repo_root": repo_root.display().to_string(),
        "success": output.status.success(),
        "evidence_artifact": "artifacts/tutorials/ingest-evidence.json",
        "log_artifact": "artifacts/tutorials/ingest.log"
    });
    let rendered = emit_tutorial_output(args, &payload, None)?;
    Ok((rendered, if output.status.success() { 0 } else { 1 }))
}

fn run_tutorials_dataset_integrity_check(
    args: &TutorialsCommandArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let sha_file_path = repo_root.join("tutorials/contracts/atlas-example-minimal.sha256");
    let pairs = load_sha256_pairs(&sha_file_path)?;
    let mut missing = Vec::new();
    let mut mismatched = Vec::new();
    for (expected, file) in pairs {
        let candidate = repo_root.join(file.as_str());
        if !candidate.exists() {
            missing.push(file);
            continue;
        }
        let bytes = fs::read(&candidate)
            .map_err(|err| format!("failed to read {}: {err}", candidate.display()))?;
        let digest = sha256_hex(&bytes);
        if digest != expected {
            mismatched.push(format!("{file}: expected {expected}, got {digest}"));
        }
    }
    let success = missing.is_empty() && mismatched.is_empty();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "dataset-integrity-check",
        "success": success,
        "missing_files": missing,
        "mismatched_hashes": mismatched
    });
    let rendered = emit_tutorial_output(args, &payload, None)?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_reproducibility_check(
    args: &TutorialsCommandArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let files = load_files_from_sha256_manifest(
        &repo_root.join("tutorials/contracts/atlas-example-minimal.sha256"),
    )?;
    let run_dir = repo_root.join("artifacts/tutorials/reproducibility");
    fs::create_dir_all(&run_dir)
        .map_err(|err| format!("failed to create {}: {err}", run_dir.display()))?;
    let run_one = run_dir.join("atlas-example-minimal.run1.tar");
    let run_two = run_dir.join("atlas-example-minimal.run2.tar");
    build_deterministic_tar(&repo_root, &files, &run_one, true)?;
    build_deterministic_tar(&repo_root, &files, &run_two, true)?;
    let run_one_hash = sha256_hex(
        &fs::read(&run_one)
            .map_err(|err| format!("failed to read {}: {err}", run_one.display()))?,
    );
    let run_two_hash = sha256_hex(
        &fs::read(&run_two)
            .map_err(|err| format!("failed to read {}: {err}", run_two.display()))?,
    );
    let reproducible = run_one_hash == run_two_hash;
    let file_level_diffs = if reproducible {
        Vec::new()
    } else {
        files
            .iter()
            .map(|file| {
                let candidate = repo_root.join(file.as_str());
                let digest = fs::read(&candidate)
                    .map(|bytes| sha256_hex(&bytes))
                    .unwrap_or_else(|_| "missing".to_string());
                format!("{file}: {digest}")
            })
            .collect::<Vec<_>>()
    };
    let evidence = serde_json::json!({
        "schema_version": 1,
        "kind": "tutorial_reproducibility_evidence",
        "run_one": {"path": run_one.display().to_string(), "sha256": run_one_hash},
        "run_two": {"path": run_two.display().to_string(), "sha256": run_two_hash},
        "reproducible": reproducible,
        "file_level_diffs": file_level_diffs
    });
    write_tutorial_report(&repo_root, "reproducibility-evidence.json", &evidence)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "reproducibility-check",
        "text": "two deterministic package runs compared",
        "reproducible": reproducible,
        "evidence_artifact": "artifacts/tutorials/reproducibility-evidence.json"
    });
    let rendered = emit_tutorial_output(args, &payload, None)?;
    Ok((rendered, if reproducible { 0 } else { 1 }))
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
    let rendered = emit_tutorial_output(&args.common, &payload, None)?;
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
    let rendered = emit_tutorial_output(args, &payload, None)?;
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
    let rendered = emit_tutorial_output(args, &payload, None)?;
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
    let rendered = emit_tutorial_output(args, &payload, None)?;
    Ok((rendered, if success { 0 } else { 1 }))
}

fn run_tutorials_contracts_explain(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let assets = load_tutorial_assets(&repo_root)?;
    let required_keys = assets
        .dataset_contract
        .get("required")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "contracts-explain",
        "contract_path": "tutorials/contracts/tutorial-dataset-contract.json",
        "required_keys": required_keys,
        "properties": assets.dataset_contract.get("properties").cloned().unwrap_or(serde_json::json!({}))
    });
    let rendered = emit_tutorial_output(args, &payload, None)?;
    Ok((rendered, 0))
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
    write_tutorial_markdown_report(
        &repo_root,
        "inventory-health.md",
        &render_tutorial_summary_markdown("inventory-health", &report),
    )?;
    let rendered = emit_tutorial_output(
        args,
        &report,
        Some(render_tutorial_summary_markdown(
            "inventory-health",
            &report,
        )),
    )?;
    Ok((rendered, 0))
}

fn load_tutorial_assets(repo_root: &Path) -> Result<TutorialAssets, String> {
    let dataset_contract_path =
        repo_root.join("tutorials/contracts/tutorial-dataset-contract.json");
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
    for entry in
        fs::read_dir(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?
    {
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
    let expected = [
        "tutorial-dataset-contract.json",
        "atlas-example-minimal.sha256",
    ];
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
    for entry in
        fs::read_dir(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read entry: {err}"))?;
        let p = entry.path();
        if p.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&p)
            .map_err(|err| format!("failed to read {}: {err}", p.display()))?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|err| format!("failed to parse {}: {err}", p.display()))?;
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

fn validate_dashboards(dashboards: &[(String, serde_json::Value)]) -> (bool, serde_json::Value) {
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

fn validate_evidence(evidence: &[(String, serde_json::Value)]) -> (bool, serde_json::Value) {
    let mut violations = Vec::new();
    for (name, item) in evidence {
        let has_id = item.get("id").and_then(|v| v.as_str()).is_some();
        let has_summary = item.get("summary").is_some() || item.get("metrics").is_some();
        if !has_id || !has_summary {
            violations.push(format!("{name}: missing required evidence keys"));
            continue;
        }
        if item.get("summary").is_some_and(|v| !v.is_string()) {
            violations.push(format!("{name}: `summary` must be a string"));
        }
        if item.get("metrics").is_some_and(|v| !v.is_object()) {
            violations.push(format!("{name}: `metrics` must be an object when provided"));
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
    let mut violations = Vec::new();
    let required_keys = contract
        .get("required")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    for key in [
        "dataset_id",
        "schema_version",
        "record_count",
        "description",
    ] {
        if !required_keys.iter().any(|found| found == key) {
            violations.push(serde_json::json!({
                "pointer": "/required",
                "message": format!("dataset contract missing required key `{key}`")
            }));
        }
    }
    if contract.get("properties").is_none() {
        violations.push(serde_json::json!({
            "pointer": "/properties",
            "message": "dataset contract must declare `properties`"
        }));
    }
    let schema_ok = contract
        .get("$schema")
        .and_then(|v| v.as_str())
        .is_some_and(|v| v.contains("json-schema.org"));
    if !schema_ok {
        violations.push(serde_json::json!({
            "pointer": "/$schema",
            "message": "dataset contract must declare a JSON Schema URI"
        }));
    }
    (
        violations.is_empty(),
        serde_json::json!({
            "required_keys": required_keys,
            "violations": violations
        }),
    )
}

fn write_tutorial_report(
    repo_root: &Path,
    file_name: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let path = repo_root.join("artifacts/tutorials").join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(payload)
        .map_err(|err| format!("serialize report failed: {err}"))?;
    fs::write(&path, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn write_tutorial_markdown_report(
    repo_root: &Path,
    file_name: &str,
    markdown: &str,
) -> Result<(), String> {
    let path = repo_root.join("artifacts/tutorials").join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&path, format!("{markdown}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn write_tutorial_log(
    repo_root: &Path,
    file_name: &str,
    stdout: &str,
    stderr: &str,
) -> Result<(), String> {
    let path = repo_root.join("artifacts/tutorials").join(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let content = format!("## stdout\n{stdout}\n\n## stderr\n{stderr}\n");
    fs::write(&path, content).map_err(|err| format!("failed to write {}: {err}", path.display()))
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

fn run_timed_step<F>(name: &str, action: F) -> serde_json::Value
where
    F: FnOnce() -> Result<(), String>,
{
    let started = Instant::now();
    let row = step_result(name, action());
    let mut object = row.as_object().cloned().unwrap_or_default();
    object.insert(
        "duration_ms".to_string(),
        serde_json::Value::from(started.elapsed().as_millis() as u64),
    );
    serde_json::Value::Object(object)
}

fn run_tutorials_workspace_setup(args: &TutorialsCommandArgs) -> Result<(), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let workspace = TutorialWorkspaceManager::new(&repo_root);
    workspace.ensure()?;
    Ok(())
}

fn run_tutorials_query_probe(args: &TutorialsCommandArgs) -> Result<(), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let output = ProcessCommand::new("cargo")
        .current_dir(repo_root)
        .args([
            "run",
            "-q",
            "-p",
            "bijux-dev-atlas",
            "--",
            "datasets",
            "validate",
            "--format",
            "json",
        ])
        .output()
        .map_err(|err| format!("failed to run datasets validate: {err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err("datasets validate command failed during tutorial query probe".to_string())
    }
}

fn render_workflow_nextest(steps: &[serde_json::Value], color: bool, verbose: bool) -> String {
    let total = steps.len();
    let mut lines = vec!["tutorials workflow".to_string()];
    let mut skipped = 0usize;
    for (index, row) in steps.iter().enumerate() {
        let success = row["success"].as_bool().unwrap_or(false);
        let is_skipped = row["skipped"].as_bool().unwrap_or(false);
        let style = if is_skipped {
            skipped += 1;
            LineStyle::Skip
        } else if success {
            LineStyle::Pass
        } else {
            LineStyle::Fail
        };
        let duration = row["duration_ms"].as_u64().unwrap_or(0);
        let name = row["name"].as_str().unwrap_or("unknown");
        let mut line =
            render_status_line(style, color, duration, index + 1, total, "tutorials", name);
        if !is_skipped {
            if let Some(error) = row["error"].as_str() {
                line.push_str(&format!(" ({error})"));
            }
        }
        lines.push(line);
        if verbose {
            lines.push(format!("  detail: step={} duration_ms={}", name, duration));
        }
    }
    let failed = steps
        .iter()
        .filter(|row| {
            let success = row["success"].as_bool().unwrap_or(false);
            let is_skipped = row["skipped"].as_bool().unwrap_or(false);
            !success && !is_skipped
        })
        .count();
    let passed = total.saturating_sub(failed).saturating_sub(skipped);
    lines.push(format!(
        "summary: total={} passed={} failed={} skipped={}",
        total, passed, failed, skipped
    ));
    lines.join("\n")
}

fn render_tutorial_summary_markdown(action: &str, payload: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_string());
    format!("# Tutorials {action}\n\n```json\n{pretty}\n```")
}

fn render_tutorial_explain_markdown() -> String {
    [
        "# Tutorials Automation",
        "",
        "- Automation engine: `bijux-dev-atlas`",
        "- Primary workflow: `bijux-dev-atlas tutorials run workflow`",
        "- Validation entrypoint: `bijux-dev-atlas tutorials verify`",
    ]
    .join("\n")
}

fn emit_tutorial_output(
    args: &TutorialsCommandArgs,
    payload: &serde_json::Value,
    markdown: Option<String>,
) -> Result<String, String> {
    if args.markdown {
        let content =
            markdown.unwrap_or_else(|| render_tutorial_summary_markdown("report", payload));
        if let Some(path) = &args.out {
            fs::write(path, format!("{content}\n"))
                .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        }
        return Ok(content);
    }
    let rendered = emit_payload(args.format, args.out.clone(), payload)?;
    if args.quiet {
        Ok(String::new())
    } else {
        Ok(rendered)
    }
}

fn load_sha256_pairs(path: &Path) -> Result<Vec<(String, String)>, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let mut rows = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let digest = parts
            .next()
            .ok_or_else(|| format!("invalid sha256 line {} in {}", line_no + 1, path.display()))?;
        let file = parts
            .next()
            .ok_or_else(|| format!("invalid sha256 line {} in {}", line_no + 1, path.display()))?;
        rows.push((digest.to_string(), file.to_string()));
    }
    rows.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(rows)
}

fn load_files_from_sha256_manifest(path: &Path) -> Result<Vec<String>, String> {
    let mut files = load_sha256_pairs(path)?
        .into_iter()
        .map(|(_, file)| file)
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    Ok(files)
}

fn build_deterministic_tar(
    repo_root: &Path,
    files: &[String],
    output: &Path,
    stable_timestamps: bool,
) -> Result<(), String> {
    let file = fs::File::create(output)
        .map_err(|err| format!("failed to create {}: {err}", output.display()))?;
    let mut builder = tar::Builder::new(file);
    for rel in files {
        let src = if Path::new(rel).is_absolute() {
            PathBuf::from(rel)
        } else {
            repo_root.join(rel)
        };
        let data =
            fs::read(&src).map_err(|err| format!("failed to read {}: {err}", src.display()))?;
        let archive_path = src
            .strip_prefix(repo_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                src.file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or("dataset-entry")
                    .to_string()
            });
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        let dynamic_mtime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("failed to read system clock: {err}"))?
            .as_secs();
        header.set_mtime(if stable_timestamps { 0 } else { dynamic_mtime });
        header.set_cksum();
        builder
            .append_data(&mut header, archive_path, data.as_slice())
            .map_err(|err| format!("failed to add {} to tar: {err}", src.display()))?;
    }
    builder
        .finish()
        .map_err(|err| format!("failed to finalize tar: {err}"))
}

fn update_sha256_manifest(
    repo_root: &Path,
    files: &[String],
    manifest: &Path,
) -> Result<(), String> {
    let mut lines = Vec::new();
    for rel in files {
        let path = if Path::new(rel).is_absolute() {
            PathBuf::from(rel)
        } else {
            repo_root.join(rel)
        };
        let digest = sha256_hex(
            &fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        );
        lines.push(format!("{digest}  {}", path.display()));
    }
    lines.sort();
    fs::write(manifest, format!("{}\n", lines.join("\n")))
        .map_err(|err| format!("failed to write {}: {err}", manifest.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct RealDataRunCatalog {
    schema_version: u64,
    runs: Vec<RealDataRun>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct RealDataRun {
    id: String,
    run_label: String,
    dataset: String,
    dataset_size_tier: String,
    ingest_mode: String,
    expected_outputs: Vec<String>,
    input_provenance: RealDataInputProvenance,
    expected_query_set: Vec<RealDataNamedQuery>,
    expected_artifacts: Vec<String>,
    expected_resource_profile: RealDataResourceProfile,
    expected_runtime_compatibility: RealDataRuntimeCompatibility,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct RealDataInputProvenance {
    url: String,
    retrieval_method: String,
    license_note: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct RealDataNamedQuery {
    name: String,
    description: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct RealDataResourceProfile {
    cpu_mem_class: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct RealDataRuntimeCompatibility {
    min_version: String,
    max_version: String,
}

fn run_tutorials_real_data_list(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let catalog = load_real_data_runs_catalog(&repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "real-data-list",
        "runs": catalog.runs.iter().map(|run| serde_json::json!({
            "id": run.id,
            "run_label": run.run_label,
            "dataset": run.dataset,
            "dataset_size_tier": run.dataset_size_tier,
            "ingest_mode": run.ingest_mode
        })).collect::<Vec<_>>()
    });
    Ok((emit_tutorial_output(args, &payload, None)?, 0))
}

fn run_tutorials_real_data_plan(args: &TutorialsRealDataPlanArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let catalog = load_real_data_runs_catalog(&repo_root)?;
    let run = catalog
        .runs
        .iter()
        .find(|run| run.id == args.run_id)
        .ok_or_else(|| format!("unknown real-data run id `{}`", args.run_id))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "real-data-plan",
        "run": run,
        "plan": [
            {"step": "fetch-dataset", "artifact": format!("artifacts/tutorials/cache/{}/dataset.bin", run.dataset)},
            {"step": "write-checksums", "artifact": format!("artifacts/tutorials/cache/{}/sha256sums.txt", run.dataset)},
            {"step": "write-provenance", "artifact": format!("artifacts/tutorials/cache/{}/provenance.json", run.dataset)},
            {"step": "ingest", "artifact": format!("artifacts/tutorials/runs/{}/ingest-report.json", run.id)}
        ]
    });
    Ok((emit_tutorial_output(&args.common, &payload, None)?, 0))
}

fn run_tutorials_real_data_fetch(args: &TutorialsRealDataRunArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let catalog = load_real_data_runs_catalog(&repo_root)?;
    let run = catalog
        .runs
        .iter()
        .find(|run| run.id == args.run_id)
        .ok_or_else(|| format!("unknown real-data run id `{}`", args.run_id))?;
    let cache_dir = repo_root
        .join("artifacts/tutorials/cache")
        .join(&run.dataset);
    fs::create_dir_all(&cache_dir)
        .map_err(|err| format!("failed to create {}: {err}", cache_dir.display()))?;
    let dataset_path = cache_dir.join("dataset.bin");
    let dataset_bytes = format!("dataset={}\nsource={}\n", run.dataset, run.input_provenance.url);
    fs::write(&dataset_path, dataset_bytes.as_bytes())
        .map_err(|err| format!("failed to write {}: {err}", dataset_path.display()))?;
    let digest = sha256_hex(dataset_bytes.as_bytes());
    let sha_path = cache_dir.join("sha256sums.txt");
    fs::write(
        &sha_path,
        format!("{digest}  {}\n", dataset_path.display()),
    )
    .map_err(|err| format!("failed to write {}: {err}", sha_path.display()))?;
    let provenance_path = cache_dir.join("provenance.json");
    let provenance = serde_json::json!({
        "schema_version": 1,
        "dataset": run.dataset,
        "run_id": run.id,
        "url": run.input_provenance.url,
        "retrieval_method": run.input_provenance.retrieval_method,
        "license_note": run.input_provenance.license_note
    });
    fs::write(
        &provenance_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&provenance)
                .map_err(|err| format!("failed to encode provenance json: {err}"))?
        ),
    )
    .map_err(|err| format!("failed to write {}: {err}", provenance_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "real-data-fetch",
        "run_id": run.id,
        "dataset": run.dataset,
        "cache_dir": cache_dir.display().to_string(),
        "artifacts": {
            "dataset_file": dataset_path.display().to_string(),
            "sha256_manifest": sha_path.display().to_string(),
            "provenance_json": provenance_path.display().to_string()
        }
    });
    Ok((emit_tutorial_output(&args.common, &payload, None)?, 0))
}

fn run_tutorials_real_data_ingest(args: &TutorialsRealDataRunArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let catalog = load_real_data_runs_catalog(&repo_root)?;
    let run = catalog
        .runs
        .iter()
        .find(|run| run.id == args.run_id)
        .ok_or_else(|| format!("unknown real-data run id `{}`", args.run_id))?;
    let cache_dir = repo_root
        .join("artifacts/tutorials/cache")
        .join(&run.dataset);
    let dataset_path = cache_dir.join("dataset.bin");
    if !dataset_path.exists() {
        return Err(format!(
            "dataset cache missing for `{}`; run `tutorials real-data fetch --run-id {}` first",
            run.id, run.id
        ));
    }
    let run_dir = repo_root.join("artifacts/tutorials/runs").join(&run.id);
    fs::create_dir_all(&run_dir)
        .map_err(|err| format!("failed to create {}: {err}", run_dir.display()))?;
    let ingest_report = serde_json::json!({
        "schema_version": 1,
        "run_id": run.id,
        "dataset": run.dataset,
        "profile": args.profile,
        "status": "ok",
        "ingest_mode": run.ingest_mode,
        "expected_outputs": run.expected_outputs,
        "runtime_compatibility": run.expected_runtime_compatibility
    });
    let ingest_report_path = run_dir.join("ingest-report.json");
    fs::write(
        &ingest_report_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&ingest_report)
                .map_err(|err| format!("failed to encode ingest report: {err}"))?
        ),
    )
    .map_err(|err| format!("failed to write {}: {err}", ingest_report_path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "real-data-ingest",
        "run_id": run.id,
        "profile": args.profile,
        "ingest_report": ingest_report_path.display().to_string()
    });
    Ok((emit_tutorial_output(&args.common, &payload, None)?, 0))
}

fn run_tutorials_real_data_doctor(args: &TutorialsCommandArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let catalog = load_real_data_runs_catalog(&repo_root)?;
    let mut rows = Vec::new();
    for run in &catalog.runs {
        let cache_dir = repo_root.join("artifacts/tutorials/cache").join(&run.dataset);
        rows.push(serde_json::json!({
            "run_id": run.id,
            "dataset": run.dataset,
            "cache_present": cache_dir.join("dataset.bin").exists(),
            "sha_present": cache_dir.join("sha256sums.txt").exists(),
            "provenance_present": cache_dir.join("provenance.json").exists()
        }));
    }
    let ok = rows.iter().all(|row| {
        row["cache_present"].as_bool().unwrap_or(false)
            && row["sha_present"].as_bool().unwrap_or(false)
            && row["provenance_present"].as_bool().unwrap_or(false)
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "domain": "tutorials",
        "action": "real-data-doctor",
        "ok": ok,
        "rows": rows
    });
    Ok((emit_tutorial_output(args, &payload, None)?, if ok { 0 } else { 2 }))
}

fn load_real_data_runs_catalog(repo_root: &Path) -> Result<RealDataRunCatalog, String> {
    let path = repo_root.join("configs/tutorials/real-data-runs.json");
    let raw =
        fs::read_to_string(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let catalog: RealDataRunCatalog = serde_json::from_str(&raw)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    validate_real_data_runs_catalog(&catalog)?;
    Ok(catalog)
}

fn validate_real_data_runs_catalog(catalog: &RealDataRunCatalog) -> Result<(), String> {
    if catalog.schema_version != 1 {
        return Err("real-data-runs schema_version must be 1".to_string());
    }
    if catalog.runs.len() != 10 {
        return Err(format!(
            "real-data-runs must contain exactly 10 runs; found {}",
            catalog.runs.len()
        ));
    }
    let mut ids = std::collections::BTreeSet::new();
    for run in &catalog.runs {
        if !ids.insert(run.id.clone()) {
            return Err(format!("duplicate real-data run id `{}`", run.id));
        }
        if run.run_label.trim().is_empty() {
            return Err(format!("run `{}` missing run_label", run.id));
        }
        if run.input_provenance.url.trim().is_empty()
            || run.input_provenance.retrieval_method.trim().is_empty()
            || run.input_provenance.license_note.trim().is_empty()
        {
            return Err(format!(
                "run `{}` input_provenance requires url, retrieval_method, and license_note",
                run.id
            ));
        }
        if run.expected_query_set.is_empty() {
            return Err(format!("run `{}` expected_query_set must be non-empty", run.id));
        }
        if run
            .expected_query_set
            .iter()
            .any(|query| query.name.trim().is_empty())
        {
            return Err(format!(
                "run `{}` expected_query_set entries require query names",
                run.id
            ));
        }
        if run.expected_artifacts.is_empty() {
            return Err(format!("run `{}` expected_artifacts must be non-empty", run.id));
        }
        if run.expected_resource_profile.cpu_mem_class.trim().is_empty() {
            return Err(format!(
                "run `{}` expected_resource_profile.cpu_mem_class is required",
                run.id
            ));
        }
        if run
            .expected_runtime_compatibility
            .min_version
            .trim()
            .is_empty()
            || run
                .expected_runtime_compatibility
                .max_version
                .trim()
                .is_empty()
        {
            return Err(format!(
                "run `{}` expected_runtime_compatibility requires min_version and max_version",
                run.id
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn dataset_contract_validator_accepts_valid_contract() {
        let contract = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "required": ["dataset_id", "schema_version", "record_count", "description"],
            "properties": {
                "dataset_id": {"type": "string"},
                "schema_version": {"type": "string"},
                "record_count": {"type": "integer"},
                "description": {"type": "string"}
            }
        });
        let (ok, detail) = validate_dataset_contract_semantics(&contract);
        assert!(ok, "expected valid contract, got {detail}");
    }

    #[test]
    fn deterministic_packager_stable_mode_has_equal_hashes() {
        let dir = tempdir().expect("tempdir");
        let repo = dir.path();
        fs::create_dir_all(repo.join("data")).expect("mkdir");
        fs::write(repo.join("data/b.txt"), "beta\n").expect("write");
        fs::write(repo.join("data/a.txt"), "alpha\n").expect("write");
        let files = vec!["data/b.txt".to_string(), "data/a.txt".to_string()];
        let out1 = repo.join("one.tar");
        let out2 = repo.join("two.tar");
        build_deterministic_tar(repo, &files, &out1, true).expect("package one");
        build_deterministic_tar(repo, &files, &out2, true).expect("package two");
        let h1 = sha256_hex(&fs::read(out1).expect("read one"));
        let h2 = sha256_hex(&fs::read(out2).expect("read two"));
        assert_eq!(h1, h2, "stable packaging must be deterministic");
    }

    #[test]
    fn sha256_manifest_roundtrip_is_consistent() {
        let dir = tempdir().expect("tempdir");
        let repo = dir.path();
        fs::create_dir_all(repo.join("data")).expect("mkdir");
        fs::write(repo.join("data/sample.json"), "{\"ok\":true}\n").expect("write");
        let manifest = repo.join("atlas.sha256");
        let files = vec!["data/sample.json".to_string()];
        update_sha256_manifest(repo, &files, &manifest).expect("update manifest");
        let pairs = load_sha256_pairs(&manifest).expect("load pairs");
        assert_eq!(pairs.len(), 1);
        assert!(
            pairs[0].1.ends_with("data/sample.json"),
            "expected manifest path to end with data/sample.json, got {}",
            pairs[0].1
        );
        let digest = sha256_hex(&fs::read(repo.join("data/sample.json")).expect("read"));
        assert_eq!(pairs[0].0, digest);
    }

    #[test]
    fn dashboard_validator_rejects_missing_panels() {
        let dashboards = vec![(
            "bad.json".to_string(),
            serde_json::json!({
                "title": "Broken"
            }),
        )];
        let (ok, detail) = validate_dashboards(&dashboards);
        assert!(!ok);
        assert_eq!(detail["checked"], 1);
        assert!(detail["violations"]
            .as_array()
            .expect("violations")
            .iter()
            .any(|v| v.as_str().unwrap_or_default().contains("missing required")));
    }

    #[test]
    fn evidence_validator_rejects_invalid_shape() {
        let evidence = vec![(
            "bad.json".to_string(),
            serde_json::json!({
                "id": 12,
                "metrics": "not-an-object"
            }),
        )];
        let (ok, detail) = validate_evidence(&evidence);
        assert!(!ok);
        assert_eq!(detail["checked"], 1);
    }

    #[test]
    fn workflow_step_reports_failure_status_and_error() {
        let row = run_timed_step("ingest", || Err("boom".to_string()));
        assert_eq!(row["name"], "ingest");
        assert_eq!(row["success"], false);
        assert_eq!(row["error"], "boom");
        assert!(row["duration_ms"].as_u64().is_some());
    }

    #[test]
    fn workflow_step_reports_success_status() {
        let row = run_timed_step("verify", || Ok(()));
        assert_eq!(row["name"], "verify");
        assert_eq!(row["success"], true);
        assert!(row.get("error").is_none());
    }

    #[test]
    fn workflow_output_mixed_status_golden() {
        let steps = vec![
            serde_json::json!({"name":"setup","success":true,"duration_ms":1}),
            serde_json::json!({"name":"ingest","success":false,"error":"fail","duration_ms":2}),
            serde_json::json!({"name":"query","success":false,"skipped":true,"duration_ms":0}),
        ];
        let text = render_workflow_nextest(&steps, false, false);
        assert!(text.contains("PASS"));
        assert!(text.contains("FAIL"));
        assert!(text.contains("SKIP"));
        assert!(text.contains("summary: total=3 passed=1 failed=1 skipped=1"));
    }

    #[test]
    fn workflow_output_long_name_golden() {
        let steps = vec![serde_json::json!({
            "name":"validate-dashboard-and-evidence-with-very-long-step-name",
            "success":true,
            "duration_ms":4
        })];
        let text = render_workflow_nextest(&steps, false, false);
        assert!(text.contains("validate-dashboard-and-evidence-with-very-long-step-name"));
    }

    #[test]
    fn workflow_output_counter_width_golden() {
        let steps = (0..12)
            .map(|i| {
                serde_json::json!({
                    "name": format!("step-{i}"),
                    "success": true,
                    "duration_ms": 1
                })
            })
            .collect::<Vec<_>>();
        let text = render_workflow_nextest(&steps, false, false);
        assert!(
            text.contains("(10/12)"),
            "counter should include multi-digit index"
        );
        assert!(
            text.contains("(12/12)"),
            "counter should include final index"
        );
    }

    #[test]
    fn workflow_output_no_ansi_golden() {
        let steps = vec![serde_json::json!({"name":"setup","success":true,"duration_ms":1})];
        let text = render_workflow_nextest(&steps, false, false);
        assert!(!text.contains('\u{1b}'));
    }
}
