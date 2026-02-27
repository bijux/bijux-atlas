// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

fn rel(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn violation(contract_id: &str, test_id: &str, path: &Path, repo_root: &Path, message: &str) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: Some(rel(path, repo_root)),
        line: Some(1),
        message: message.to_string(),
        evidence: None,
    }
}

fn read_target_registry(repo_root: &Path) -> Result<Value, String> {
    let path = repo_root.join("make/target-list.json");
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {} failed: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {} failed: {e}", path.display()))
}

fn test_make_000_allowed_surface(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-000";
    let test_id = "make.surface.allowed_files";
    let make_root = ctx.repo_root.join("make");
    let allowed = BTreeSet::from([
        "CONTRACT.mk",
        "README.md",
        "checks.mk",
        "env.mk",
        "help.md",
        "help.mk",
        "internal.mk",
        "paths.mk",
        "phony.mk",
        "public.mk",
        "target-list.json",
        "vars.mk",
    ]);
    let mut violations = Vec::new();
    let Ok(entries) = std::fs::read_dir(&make_root) else {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("make".to_string()),
            line: Some(1),
            message: "make/ directory is missing".to_string(),
            evidence: None,
        }]);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if !allowed.contains(name) {
                violations.push(violation(contract_id, test_id, &path, &ctx.repo_root, "unexpected root file under make/"));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_001_target_registry_valid(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-001";
    let test_id = "make.registry.target_list_valid";
    let path = ctx.repo_root.join("make/target-list.json");
    let registry = match read_target_registry(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&path, &ctx.repo_root)),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    let public_targets = registry
        .get("public_targets")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    if public_targets.is_empty() {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(rel(&path, &ctx.repo_root)),
            line: Some(1),
            message: "make target registry has no public targets".to_string(),
            evidence: None,
        }]);
    }
    TestResult::Pass
}

fn test_make_002_root_makefile_delegates(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-002";
    let test_id = "make.root.includes_curated_wrapper";
    let path = ctx.repo_root.join("Makefile");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("Makefile".to_string()),
            line: Some(1),
            message: "Makefile is missing".to_string(),
            evidence: None,
        }]);
    };
    if text.contains("include make/public.mk") || text.contains("include make/makefiles/root.mk") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            &path,
            &ctx.repo_root,
            "Makefile must include make/makefiles/root.mk",
        )])
    }
}

fn test_make_003_public_wrappers_use_control_plane(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-003";
    let test_id = "make.targets.delegate_to_control_plane";
    let registry_path = ctx.repo_root.join("make/target-list.json");
    let registry = match read_target_registry(&ctx.repo_root) {
        Ok(value) => value,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&registry_path, &ctx.repo_root)),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    let root_path = ctx.repo_root.join("make/makefiles/root.mk");
    let Ok(root_text) = std::fs::read_to_string(&root_path) else {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(rel(&root_path, &ctx.repo_root)),
            line: Some(1),
            message: "make/makefiles/root.mk is missing".to_string(),
            evidence: None,
        }]);
    };
    let curated = registry
        .get("public_targets")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let mut violations = Vec::new();
    for target in curated {
        if !root_text.contains(&target) {
            violations.push(violation(
                contract_id,
                test_id,
                &root_path,
                &ctx.repo_root,
                &format!("public make target `{target}` is missing from curated root wrapper"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    Ok(vec![
        Contract {
            id: ContractId("MAKE-000".to_string()),
            title: "make directory surface",
            tests: vec![TestCase {
                id: TestId("make.surface.allowed_files".to_string()),
                title: "make root contains only curated wrapper files",
                kind: TestKind::Pure,
                run: test_make_000_allowed_surface,
            }],
        },
        Contract {
            id: ContractId("MAKE-001".to_string()),
            title: "make target registry",
            tests: vec![TestCase {
                id: TestId("make.registry.target_list_valid".to_string()),
                title: "make target registry exists and declares public targets",
                kind: TestKind::Pure,
                run: test_make_001_target_registry_valid,
            }],
        },
        Contract {
            id: ContractId("MAKE-002".to_string()),
            title: "make root wrapper include",
            tests: vec![TestCase {
                id: TestId("make.root.includes_curated_wrapper".to_string()),
                title: "Makefile delegates to the curated root wrapper",
                kind: TestKind::Pure,
                run: test_make_002_root_makefile_delegates,
            }],
        },
        Contract {
            id: ContractId("MAKE-003".to_string()),
            title: "make delegated target wrappers",
            tests: vec![TestCase {
                id: TestId("make.targets.delegate_to_control_plane".to_string()),
                title: "make wrapper recipes delegate through the control-plane",
                kind: TestKind::Pure,
                run: test_make_003_public_wrappers_use_control_plane,
            }],
        },
    ])
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "MAKE-000" => "Keep the make/ tree constrained to curated wrapper sources and inventories."
            .to_string(),
        "MAKE-001" => "Require a deterministic public target registry so the make surface is inspectable."
            .to_string(),
        "MAKE-002" => "Makefile must route through the curated root wrapper instead of growing independent logic."
            .to_string(),
        "MAKE-003" => "Public make recipes must delegate to the Rust control-plane rather than embedding operational logic."
            .to_string(),
        _ => "Unknown make contract id.".to_string(),
    }
}

pub fn contract_gate_command(contract_id: &str) -> &'static str {
    match contract_id {
        "MAKE-000" | "MAKE-001" | "MAKE-002" | "MAKE-003" => "bijux dev atlas contracts make --mode static",
        _ => "bijux dev atlas contracts make --mode static",
    }
}
