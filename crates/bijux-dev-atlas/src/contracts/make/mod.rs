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

fn sorted_makefiles(repo_root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let roots = [
        repo_root.join("Makefile"),
        repo_root.join("make"),
        repo_root.join("make/makefiles"),
    ];
    for root in roots {
        if root.is_file() {
            files.push(root);
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        let mut paths = entries.flatten().map(|entry| entry.path()).collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if path.extension().and_then(|value| value.to_str()) == Some("mk")
                || path.file_name().and_then(|value| value.to_str()) == Some("Makefile")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    files.dedup();
    files
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
    let targets = public_targets
        .iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let unique = targets.iter().cloned().collect::<BTreeSet<_>>();
    if unique.len() != targets.len() {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(rel(&path, &ctx.repo_root)),
            line: Some(1),
            message: "make target registry must not contain duplicate public targets".to_string(),
            evidence: None,
        }]);
    }
    let mut sorted = targets.clone();
    sorted.sort();
    if sorted != targets {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(rel(&path, &ctx.repo_root)),
            line: Some(1),
            message: "make target registry public_targets must be sorted".to_string(),
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

fn test_make_004_help_target_is_curated(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-004";
    let test_id = "make.targets.help_is_public";
    let registry_path = ctx.repo_root.join("make/target-list.json");
    let root_path = ctx.repo_root.join("make/makefiles/root.mk");
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
    let targets = registry
        .get("public_targets")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    if !targets.contains("help") {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(rel(&registry_path, &ctx.repo_root)),
            line: Some(1),
            message: "make target registry must include help".to_string(),
            evidence: None,
        });
    }
    if !root_text.contains("\nhelp:") {
        violations.push(violation(
            contract_id,
            test_id,
            &root_path,
            &ctx.repo_root,
            "make/makefiles/root.mk must define help target",
        ));
    }
    if !root_text.contains("Curated make targets") {
        violations.push(violation(
            contract_id,
            test_id,
            &root_path,
            &ctx.repo_root,
            "help target must print the curated public target surface",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_005_internal_targets_prefixed(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-005";
    let test_id = "make.targets.internal_prefix_policy";
    let root_path = ctx.repo_root.join("make/makefiles/root.mk");
    let phony_path = ctx.repo_root.join("make/phony.mk");
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
    let Ok(phony_text) = std::fs::read_to_string(&phony_path) else {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(rel(&phony_path, &ctx.repo_root)),
            line: Some(1),
            message: "make/phony.mk is missing".to_string(),
            evidence: None,
        }]);
    };
    let mut violations = Vec::new();
    for line in root_text.lines() {
        if line.starts_with('\t') || line.starts_with(' ') {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.contains('=') {
            continue;
        }
        if let Some((name, _)) = trimmed.split_once(':') {
            let target = name.trim();
            if target.is_empty()
                || target.starts_with('.')
                || target.contains(' ')
            {
                continue;
            }
            if target.starts_with("_internal-") {
                continue;
            }
            if target == "help"
                || target == "doctor"
                || target == "k8s-render"
                || target == "k8s-validate"
                || target == "stack-up"
                || target == "stack-down"
                || target == "ops-fast"
                || target == "ops-pr"
                || target == "ops-nightly"
            {
                continue;
            }
            violations.push(violation(
                contract_id,
                test_id,
                &root_path,
                &ctx.repo_root,
                &format!("non-public helper target `{target}` must use the `_internal-` prefix"),
            ));
        }
    }
    for token in phony_text.split_whitespace() {
        if token == ".PHONY:" || token.is_empty() {
            continue;
        }
        if token != "help-contract" && token != "make-target-list" && token != "make-contract-check" {
            violations.push(violation(
                contract_id,
                test_id,
                &phony_path,
                &ctx.repo_root,
                &format!("make/phony.mk may only expose curated helper targets, found `{token}`"),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_006_shell_is_pinned(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-006";
    let test_id = "make.runtime.shell_is_pinned";
    let root_path = ctx.repo_root.join("make/makefiles/root.mk");
    let macro_path = ctx.repo_root.join("make/makefiles/_macro.mk");
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
    let Ok(macro_text) = std::fs::read_to_string(&macro_path) else {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some(rel(&macro_path, &ctx.repo_root)),
            line: Some(1),
            message: "make/makefiles/_macro.mk is missing".to_string(),
            evidence: None,
        }]);
    };
    let mut violations = Vec::new();
    if !root_text.contains("SHELL := /bin/sh") {
        violations.push(violation(
            contract_id,
            test_id,
            &root_path,
            &ctx.repo_root,
            "make/makefiles/root.mk must pin SHELL := /bin/sh",
        ));
    }
    if !macro_text.contains("SHELL := /bin/sh") {
        violations.push(violation(
            contract_id,
            test_id,
            &macro_path,
            &ctx.repo_root,
            "make/makefiles/_macro.mk must pin SHELL := /bin/sh",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_007_include_graph_acyclic(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-007";
    let test_id = "make.includes.acyclic";
    let files = sorted_makefiles(&ctx.repo_root);
    let mut edges = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut violations = Vec::new();
    for path in &files {
        let rel_path = rel(path, &ctx.repo_root);
        let Ok(text) = std::fs::read_to_string(path) else {
            violations.push(violation(
                contract_id,
                test_id,
                path,
                &ctx.repo_root,
                "make include source must be readable",
            ));
            continue;
        };
        let mut includes = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(include_path) = trimmed.strip_prefix("include ") {
                includes.push(include_path.trim().to_string());
            }
        }
        includes.sort();
        includes.dedup();
        edges.insert(rel_path, includes);
    }
    fn visit(
        node: &str,
        edges: &std::collections::BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> Option<String> {
        if visited.contains(node) {
            return None;
        }
        if !visiting.insert(node.to_string()) {
            return Some(node.to_string());
        }
        for next in edges.get(node).into_iter().flatten() {
            if edges.contains_key(next) {
                if let Some(cycle) = visit(next, edges, visiting, visited) {
                    return Some(cycle);
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
        None
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for node in edges.keys() {
        if let Some(cycle) = visit(node, &edges, &mut visiting, &mut visited) {
            let path = ctx.repo_root.join(&cycle);
            violations.push(violation(
                contract_id,
                test_id,
                &path,
                &ctx.repo_root,
                "make include graph must be acyclic",
            ));
            break;
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_008_no_duplicate_targets(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-008";
    let test_id = "make.targets.unique";
    let files = sorted_makefiles(&ctx.repo_root);
    let mut owners = std::collections::BTreeMap::<String, String>::new();
    let mut violations = Vec::new();
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in text.lines() {
            if line.starts_with('\t') || line.starts_with(' ') {
                continue;
            }
            let trimmed = line.trim_start();
            if trimmed.contains('=') {
                continue;
            }
            let Some((name, _)) = trimmed.split_once(':') else {
                continue;
            };
            let target = name.trim();
            if target.is_empty() || target.starts_with('.') || target.contains(' ') {
                continue;
            }
            let path_rel = rel(&path, &ctx.repo_root);
            if let Some(previous) = owners.insert(target.to_string(), path_rel.clone()) {
                if previous != path_rel {
                    violations.push(violation(
                        contract_id,
                        test_id,
                        &path,
                        &ctx.repo_root,
                        &format!("target `{target}` is declared in more than one make source"),
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_009_oneshell_forbidden(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-009";
    let test_id = "make.runtime.oneshell_forbidden";
    let mut violations = Vec::new();
    for path in sorted_makefiles(&ctx.repo_root) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.lines().any(|line| line.trim() == ".ONESHELL:") {
            violations.push(violation(
                contract_id,
                test_id,
                &path,
                &ctx.repo_root,
                ".ONESHELL is forbidden in curated make wrappers",
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
        Contract {
            id: ContractId("MAKE-004".to_string()),
            title: "make help surface",
            tests: vec![TestCase {
                id: TestId("make.targets.help_is_public".to_string()),
                title: "make help stays in the curated public target registry",
                kind: TestKind::Pure,
                run: test_make_004_help_target_is_curated,
            }],
        },
        Contract {
            id: ContractId("MAKE-005".to_string()),
            title: "make internal target naming",
            tests: vec![TestCase {
                id: TestId("make.targets.internal_prefix_policy".to_string()),
                title: "internal make helpers use the approved naming boundary",
                kind: TestKind::Pure,
                run: test_make_005_internal_targets_prefixed,
            }],
        },
        Contract {
            id: ContractId("MAKE-006".to_string()),
            title: "make shell pinning",
            tests: vec![TestCase {
                id: TestId("make.runtime.shell_is_pinned".to_string()),
                title: "make shell is pinned in curated wrappers",
                kind: TestKind::Pure,
                run: test_make_006_shell_is_pinned,
            }],
        },
        Contract {
            id: ContractId("MAKE-007".to_string()),
            title: "make include graph",
            tests: vec![TestCase {
                id: TestId("make.includes.acyclic".to_string()),
                title: "make include graph is acyclic",
                kind: TestKind::Pure,
                run: test_make_007_include_graph_acyclic,
            }],
        },
        Contract {
            id: ContractId("MAKE-008".to_string()),
            title: "make unique target declarations",
            tests: vec![TestCase {
                id: TestId("make.targets.unique".to_string()),
                title: "make target names are unique across curated sources",
                kind: TestKind::Pure,
                run: test_make_008_no_duplicate_targets,
            }],
        },
        Contract {
            id: ContractId("MAKE-009".to_string()),
            title: "make oneshell policy",
            tests: vec![TestCase {
                id: TestId("make.runtime.oneshell_forbidden".to_string()),
                title: "curated make wrappers forbid .ONESHELL",
                kind: TestKind::Pure,
                run: test_make_009_oneshell_forbidden,
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
        "MAKE-004" => "Keep the `help` target in the public registry and make it print the curated surface."
            .to_string(),
        "MAKE-005" => "Reserve unprefixed targets for public entrypoints and keep helper targets behind stable internal names."
            .to_string(),
        "MAKE-006" => "Pin the shell used by curated make wrappers so runtime behavior stays deterministic."
            .to_string(),
        "MAKE-007" => "Keep the make include graph acyclic so wrapper composition remains inspectable and deterministic."
            .to_string(),
        "MAKE-008" => "Prevent duplicate target declarations across curated make sources."
            .to_string(),
        "MAKE-009" => "Forbid .ONESHELL so recipe behavior stays line-scoped and predictable."
            .to_string(),
        _ => "Unknown make contract id.".to_string(),
    }
}

pub fn contract_gate_command(contract_id: &str) -> &'static str {
    match contract_id {
        "MAKE-000" | "MAKE-001" | "MAKE-002" | "MAKE-003" | "MAKE-004" | "MAKE-005"
        | "MAKE-006" | "MAKE-007" | "MAKE-008" | "MAKE-009" => {
            "bijux dev atlas contracts make --mode static"
        }
        _ => "bijux dev atlas contracts make --mode static",
    }
}
