// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

fn violation(contract_id: &str, test_id: &str, file: Option<String>, message: impl Into<String>) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file,
        line: None,
        message: message.into(),
        evidence: None,
    }
}

fn rel(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn collect_rs_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|err| format!("read {} failed: {err}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("read_dir entry in {} failed: {err}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    walk(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn test_control_plane_001_no_legacy_dirs(ctx: &RunContext) -> TestResult {
    let mut violations = Vec::new();
    for rel_path in ["scripts", "tools", "xtask"] {
        let path = ctx.repo_root.join(rel_path);
        if path.exists() {
            violations.push(violation(
                "CONTROL-PLANE-001",
                "control_plane.surface.no_legacy_dirs",
                Some(rel_path.to_string()),
                format!("legacy automation directory `{rel_path}` must not exist"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_control_plane_002_process_boundary(ctx: &RunContext) -> TestResult {
    let files = match collect_rs_files(&ctx.repo_root.join("crates/bijux-dev-atlas/src")) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "CONTROL-PLANE-002",
                "control_plane.effects.process_boundary",
                Some("crates/bijux-dev-atlas/src".to_string()),
                err,
            )]);
        }
    };
    let allowed_prefixes = [
        "crates/bijux-dev-atlas/src/adapters/",
        "crates/bijux-dev-atlas/src/commands/",
        "crates/bijux-dev-atlas/src/contracts/",
        "crates/bijux-dev-atlas/src/core/checks/",
        "crates/bijux-dev-atlas/src/cli/dispatch.rs",
        "crates/bijux-dev-atlas/src/runtime_entry.rs",
    ];
    let needles = ["std::process::Command", "ProcessCommand::new", "Command::new("];
    let mut violations = Vec::new();
    for file in files {
        let relative = rel(&ctx.repo_root, &file);
        let text = match fs::read_to_string(&file) {
            Ok(value) => value,
            Err(err) => {
                violations.push(violation(
                    "CONTROL-PLANE-002",
                    "control_plane.effects.process_boundary",
                    Some(relative),
                    format!("read source failed: {err}"),
                ));
                continue;
            }
        };
        if !needles.iter().any(|needle| text.contains(needle)) {
            continue;
        }
        if !allowed_prefixes.iter().any(|prefix| relative.starts_with(prefix) || relative == *prefix) {
            violations.push(violation(
                "CONTROL-PLANE-002",
                "control_plane.effects.process_boundary",
                Some(relative),
                "direct subprocess invocation escaped the approved control-plane surfaces",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_control_plane_003_fs_mutation_boundary(ctx: &RunContext) -> TestResult {
    let files = match collect_rs_files(&ctx.repo_root.join("crates/bijux-dev-atlas/src")) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "CONTROL-PLANE-003",
                "control_plane.effects.fs_mutation_boundary",
                Some("crates/bijux-dev-atlas/src".to_string()),
                err,
            )]);
        }
    };
    let allowed_prefixes = [
        "crates/bijux-dev-atlas/src/adapters/",
        "crates/bijux-dev-atlas/src/cli/dispatch.rs",
        "crates/bijux-dev-atlas/src/commands/",
        "crates/bijux-dev-atlas/src/contracts/",
        "crates/bijux-dev-atlas/src/core/checks/",
        "crates/bijux-dev-atlas/src/core/ops_inventory/",
        "crates/bijux-dev-atlas/src/runtime_entry_checks_surface.rs",
        "crates/bijux-dev-atlas/src/schema_support.rs",
    ];
    let needles = [
        "fs::write(",
        "std::fs::write(",
        "fs::create_dir_all(",
        "std::fs::create_dir_all(",
        "fs::remove_dir_all(",
        "std::fs::remove_dir_all(",
        "fs::remove_file(",
        "std::fs::remove_file(",
        "fs::copy(",
        "std::fs::copy(",
    ];
    let mut violations = Vec::new();
    for file in files {
        let relative = rel(&ctx.repo_root, &file);
        let text = match fs::read_to_string(&file) {
            Ok(value) => value,
            Err(err) => {
                violations.push(violation(
                    "CONTROL-PLANE-003",
                    "control_plane.effects.fs_mutation_boundary",
                    Some(relative),
                    format!("read source failed: {err}"),
                ));
                continue;
            }
        };
        if !needles.iter().any(|needle| text.contains(needle)) {
            continue;
        }
        if !allowed_prefixes.iter().any(|prefix| relative.starts_with(prefix) || relative == *prefix) {
            violations.push(violation(
                "CONTROL-PLANE-003",
                "control_plane.effects.fs_mutation_boundary",
                Some(relative),
                "filesystem mutation escaped the approved control-plane surfaces",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_control_plane_004_help_snapshot(ctx: &RunContext) -> TestResult {
    if !ctx.allow_subprocess {
        return TestResult::Skip("help snapshot requires --allow-subprocess".to_string());
    }
    let output = match Command::new("cargo")
        .current_dir(&ctx.repo_root)
        .args(["run", "-q", "-p", "bijux-dev-atlas", "--", "--help"])
        .output()
    {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "CONTROL-PLANE-004",
                "control_plane.cli.help_snapshot",
                Some("crates/bijux-dev-atlas/tests/goldens/help.txt".to_string()),
                format!("cargo run --help failed to start: {err}"),
            )]);
        }
    };
    if !output.status.success() {
        return TestResult::Fail(vec![violation(
            "CONTROL-PLANE-004",
            "control_plane.cli.help_snapshot",
            Some("crates/bijux-dev-atlas/tests/goldens/help.txt".to_string()),
            "cargo run --help did not exit successfully",
        )]);
    }
    let actual = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let snapshot_path = ctx
        .repo_root
        .join("crates/bijux-dev-atlas/tests/goldens/help.txt");
    let expected = match fs::read_to_string(&snapshot_path) {
        Ok(value) => value.replace("\r\n", "\n"),
        Err(err) => {
            return TestResult::Fail(vec![violation(
                "CONTROL-PLANE-004",
                "control_plane.cli.help_snapshot",
                Some(rel(&ctx.repo_root, &snapshot_path)),
                format!("read snapshot failed: {err}"),
            )]);
        }
    };
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "CONTROL-PLANE-004",
            "control_plane.cli.help_snapshot",
            Some(rel(&ctx.repo_root, &snapshot_path)),
            "control-plane help output drifted from the committed snapshot",
        )])
    }
}

fn test_control_plane_005_contracts_run_id_override(ctx: &RunContext) -> TestResult {
    let surfaces = ctx.repo_root.join("crates/bijux-dev-atlas/src/cli/surfaces.rs");
    let engine = ctx.repo_root.join("crates/bijux-dev-atlas/src/contracts/engine_model.rs");
    let mut violations = Vec::new();
    let surfaces_text = fs::read_to_string(&surfaces).unwrap_or_default();
    if !surfaces_text.contains("pub run_id: Option<String>") {
        violations.push(violation(
            "CONTROL-PLANE-005",
            "control_plane.run_id.override",
            Some(rel(&ctx.repo_root, &surfaces)),
            "contracts command surface must expose --run-id",
        ));
    }
    let engine_text = fs::read_to_string(&engine).unwrap_or_default();
    if !engine_text.contains("run_id_override") {
        violations.push(violation(
            "CONTROL-PLANE-005",
            "control_plane.run_id.override",
            Some(rel(&ctx.repo_root, &engine)),
            "contracts engine metadata must honor explicit run_id overrides",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    if !repo_root.join("crates/bijux-dev-atlas/Cargo.toml").exists() {
        return Err("control-plane contracts require the bijux-dev-atlas crate".to_string());
    }
    Ok(vec![
        Contract {
            id: ContractId("CONTROL-PLANE-001".to_string()),
            title: "control-plane forbids legacy automation directories",
            tests: vec![TestCase {
                id: TestId("controlplane.surface.no_legacy_dirs".to_string()),
                title: "scripts tools and xtask directories do not exist at the repo root",
                kind: TestKind::Pure,
                run: test_control_plane_001_no_legacy_dirs,
            }],
        },
        Contract {
            id: ContractId("CONTROL-PLANE-002".to_string()),
            title: "control-plane subprocess usage stays on approved surfaces",
            tests: vec![TestCase {
                id: TestId("controlplane.effects.process_boundary".to_string()),
                title: "direct subprocess invocations stay inside approved control-plane surfaces",
                kind: TestKind::Pure,
                run: test_control_plane_002_process_boundary,
            }],
        },
        Contract {
            id: ContractId("CONTROL-PLANE-003".to_string()),
            title: "control-plane filesystem mutation stays on approved surfaces",
            tests: vec![TestCase {
                id: TestId("controlplane.effects.fs_mutation_boundary".to_string()),
                title: "filesystem mutation stays inside approved control-plane surfaces",
                kind: TestKind::Pure,
                run: test_control_plane_003_fs_mutation_boundary,
            }],
        },
        Contract {
            id: ContractId("CONTROL-PLANE-004".to_string()),
            title: "control-plane help output stays stable",
            tests: vec![TestCase {
                id: TestId("controlplane.cli.help_snapshot".to_string()),
                title: "bijux-dev-atlas --help matches the committed snapshot",
                kind: TestKind::Subprocess,
                run: test_control_plane_004_help_snapshot,
            }],
        },
        Contract {
            id: ContractId("CONTROL-PLANE-005".to_string()),
            title: "contracts surface supports explicit run ids",
            tests: vec![TestCase {
                id: TestId("controlplane.run_id.override".to_string()),
                title: "contracts command surface and engine honor explicit run_id overrides",
                kind: TestKind::Pure,
                run: test_control_plane_005_contracts_run_id_override,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "CONTROL-PLANE-001" => {
            "Legacy automation directories such as scripts, tools, and xtask are forbidden.".to_string()
        }
        "CONTROL-PLANE-002" => {
            "Direct subprocess spawning must stay inside approved control-plane surfaces.".to_string()
        }
        "CONTROL-PLANE-003" => {
            "Filesystem mutation must stay inside approved control-plane surfaces.".to_string()
        }
        "CONTROL-PLANE-004" => {
            "The bijux-dev-atlas CLI help surface is snapshot-governed.".to_string()
        }
        "CONTROL-PLANE-005" => {
            "Contracts runs must support explicit run_id overrides for deterministic reproduction.".to_string()
        }
        _ => "Fix the listed violations and rerun `bijux dev atlas contracts control-plane`.".to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts control-plane --mode static"
}
