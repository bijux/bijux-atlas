// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

mod surface_contracts;
mod wrapper_contracts;

pub(super) fn rel(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

pub(super) fn violation(
    contract_id: &str,
    test_id: &str,
    path: &Path,
    repo_root: &Path,
    message: &str,
) -> Violation {
    Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: Some(rel(path, repo_root)),
        line: Some(1),
        message: message.to_string(),
        evidence: None,
    }
}

fn top_level_make_files(repo_root: &Path) -> Vec<PathBuf> {
    let make_root = repo_root.join("make");
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(&make_root) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files
}

pub(super) fn sorted_make_sources(repo_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let roots = [repo_root.join("Makefile"), repo_root.join("make")];
    for root in roots {
        if root.is_file() {
            files.push(root);
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        let mut paths = entries
            .flatten()
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let name = path.file_name().and_then(|value| value.to_str());
            let ext = path.extension().and_then(|value| value.to_str());
            if name == Some("Makefile") || ext == Some("mk") {
                files.push(path);
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

fn include_lines(path: &Path) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let mut includes = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("include "))
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>();
    includes.sort();
    Ok(includes)
}

fn test_make_dir_001_allowed_root_docs_only(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-DIR-001";
    let test_id = "make.docs.allowed_root_docs_only";
    let allowed = BTreeSet::from(["CONTRACT.md", "README.md"]);
    let mut violations = Vec::new();
    for path in top_level_make_files(&ctx.repo_root) {
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !allowed.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                &path,
                &ctx.repo_root,
                "top-level make markdown is limited to README.md and CONTRACT.md",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_dir_002_no_nested_markdown(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-DIR-002";
    let test_id = "make.docs.no_nested_markdown";
    let make_root = ctx.repo_root.join("make");
    let mut violations = Vec::new();
    let Ok(entries) = std::fs::read_dir(&make_root) else {
        return TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("make".to_string()),
            line: Some(1),
            message: "make directory is missing".to_string(),
            evidence: None,
        }]);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(children) = std::fs::read_dir(&path) else {
            continue;
        };
        for child in children.flatten() {
            let child_path = child.path();
            if child_path.extension().and_then(|value| value.to_str()) == Some("md") {
                violations.push(violation(
                    contract_id,
                    test_id,
                    &child_path,
                    &ctx.repo_root,
                    "markdown files are forbidden under nested make directories; keep prose at make/README.md or make/CONTRACT.md only",
                ));
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_dir_003_allowed_root_files(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-DIR-003";
    let test_id = "make.surface.allowed_root_files";
    let allowed = BTreeSet::from([
        "CONTRACT.md",
        "README.md",
        "_internal.mk",
        "build.mk",
        "cargo.mk",
        "checks.mk",
        "ci.mk",
        "configs.mk",
        "contracts.mk",
        "dev.mk",
        "docker.mk",
        "docs.mk",
        "gates.mk",
        "k8s.mk",
        "macros.mk",
        "ops.mk",
        "paths.mk",
        "policies.mk",
        "public.mk",
        "root.mk",
        "runenv.mk",
        "target-list.json",
        "verification.mk",
        "vars.mk",
    ]);
    let mut violations = Vec::new();
    for path in top_level_make_files(&ctx.repo_root) {
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !allowed.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                &path,
                &ctx.repo_root,
                "unexpected top-level file under make/",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_contracts_001_single_entrypoint(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-001";
    let test_id = "make.contracts.single_entrypoint";
    let contracts_path = ctx.repo_root.join("make/contracts.mk");
    let public_path = ctx.repo_root.join("make/public.mk");
    let mut violations = Vec::new();
    if !contracts_path.is_file() {
        violations.push(violation(
            contract_id,
            test_id,
            &contracts_path,
            &ctx.repo_root,
            "make/contracts.mk must exist",
        ));
    }
    let public_text = match std::fs::read_to_string(&public_path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&public_path, &ctx.repo_root)),
                line: Some(1),
                message: format!("read make/public.mk failed: {err}"),
                evidence: None,
            }])
        }
    };
    if !public_text.contains("include make/contracts.mk") {
        violations.push(violation(
            contract_id,
            test_id,
            &public_path,
            &ctx.repo_root,
            "make/public.mk must include make/contracts.mk",
        ));
    }
    if public_text.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("contracts:") || trimmed.starts_with("contracts-")
    }) {
        violations.push(violation(
            contract_id,
            test_id,
            &public_path,
            &ctx.repo_root,
            "contracts targets must be defined only in make/contracts.mk",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_contracts_002_target_surface(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-002";
    let test_id = "make.contracts.target_surface";
    let path = ctx.repo_root.join("make/contracts.mk");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&path, &ctx.repo_root)),
                line: Some(1),
                message: format!("read make/contracts.mk failed: {err}"),
                evidence: None,
            }])
        }
    };
    let expected_public = BTreeSet::from([
        "contracts",
        "contracts-pr",
        "contracts-merge",
        "contracts-release",
        "contracts-all",
        "contracts-fast",
        "contracts-changed",
        "contracts-json",
        "contracts-ci",
        "contracts-root",
        "contracts-configs",
        "contracts-configs-required",
        "contracts-docs",
        "contracts-docs-required",
        "contracts-docker",
        "contracts-make",
        "contracts-make-required",
        "contracts-ops",
        "contracts-help",
    ]);
    let mut found_public = BTreeSet::new();
    let mut violations = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("contracts") && !trimmed.starts_with("_contracts") {
            continue;
        }
        if let Some((target, _)) = trimmed.split_once(':') {
            if target.starts_with("_contracts") {
                continue;
            }
            found_public.insert(target.to_string());
            if !expected_public.contains(target) {
                violations.push(Violation {
                    contract_id: contract_id.to_string(),
                    test_id: test_id.to_string(),
                    file: Some(rel(&path, &ctx.repo_root)),
                    line: Some(index + 1),
                    message: format!("unexpected contracts public target `{target}`"),
                    evidence: None,
                });
            }
        }
    }
    for expected in expected_public {
        if !found_public.contains(expected) {
            violations.push(Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&path, &ctx.repo_root)),
                line: Some(1),
                message: format!("missing required contracts target `{expected}`"),
                evidence: None,
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_contracts_003_delegate_only(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-003";
    let test_id = "make.contracts.delegate_only";
    let path = ctx.repo_root.join("make/contracts.mk");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&path, &ctx.repo_root)),
                line: Some(1),
                message: format!("read make/contracts.mk failed: {err}"),
                evidence: None,
            }])
        }
    };
    let mut violations = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("@$(DEV_ATLAS)") && !trimmed.starts_with("@CI=1 $(DEV_ATLAS)") {
            continue;
        }
        if !(trimmed.contains(" contracts ") || trimmed.contains(" check run --suite ")) {
            violations.push(Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&path, &ctx.repo_root)),
                line: Some(index + 1),
                message:
                    "contracts targets must delegate only to bijux-dev-atlas contracts commands or required-suite check entrypoints"
                        .to_string(),
                evidence: Some(trimmed.to_string()),
            });
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_env_001_single_macros_and_runenv(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-ENV-001";
    let test_id = "make.env.single_macros_and_runenv";
    let mut macro_paths = Vec::new();
    let mut runenv_paths = Vec::new();
    for path in sorted_make_sources(&ctx.repo_root) {
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.contains("macro") || name == "macros.mk" {
            macro_paths.push(rel(&path, &ctx.repo_root));
        }
        if name.contains("runenv") {
            runenv_paths.push(rel(&path, &ctx.repo_root));
        }
    }
    macro_paths.sort();
    runenv_paths.sort();
    let expected_macros = vec!["make/macros.mk".to_string()];
    let expected_runenv = vec!["make/runenv.mk".to_string()];
    let mut violations = Vec::new();
    if macro_paths != expected_macros {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("make".to_string()),
            line: Some(1),
            message: format!(
                "expected exactly one macros file: {expected_macros:?}, found {macro_paths:?}"
            ),
            evidence: None,
        });
    }
    if runenv_paths != expected_runenv {
        violations.push(Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("make".to_string()),
            line: Some(1),
            message: format!(
                "expected exactly one runenv file: {expected_runenv:?}, found {runenv_paths:?}"
            ),
            evidence: None,
        });
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_env_002_role_boundary(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-ENV-002";
    let test_id = "make.env.role_boundary";
    let macros_path = ctx.repo_root.join("make/macros.mk");
    let runenv_path = ctx.repo_root.join("make/runenv.mk");
    let macros_text = match std::fs::read_to_string(&macros_path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&macros_path, &ctx.repo_root)),
                line: Some(1),
                message: format!("read {} failed: {err}", macros_path.display()),
                evidence: None,
            }]);
        }
    };
    let runenv_text = match std::fs::read_to_string(&runenv_path) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&runenv_path, &ctx.repo_root)),
                line: Some(1),
                message: format!("read {} failed: {err}", runenv_path.display()),
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    for line in macros_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("export ")
            || trimmed.starts_with("include ")
            || trimmed
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            violations.push(violation(
                contract_id,
                test_id,
                &macros_path,
                &ctx.repo_root,
                "make/macros.mk must contain only pure macro helpers",
            ));
            break;
        }
    }
    let has_export = runenv_text
        .lines()
        .any(|line| line.trim_start().starts_with("export "));
    if !has_export {
        violations.push(violation(
            contract_id,
            test_id,
            &runenv_path,
            &ctx.repo_root,
            "make/runenv.mk must export deterministic environment defaults",
        ));
    }
    for line in runenv_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("export ") {
            continue;
        }
        if trimmed.contains(" =") || trimmed.contains("= ") || trimmed.contains(" = ") {
            let name = trimmed.split('=').next().unwrap_or("").trim();
            if name
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_lowercase())
            {
                violations.push(violation(
                    contract_id,
                    test_id,
                    &runenv_path,
                    &ctx.repo_root,
                    "make/runenv.mk must not define helper macros",
                ));
                break;
            }
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_include_001_root_single_entrypoint(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-INCLUDE-001";
    let test_id = "make.includes.root_single_entrypoint";
    let makefile = ctx.repo_root.join("Makefile");
    let includes = match include_lines(&makefile) {
        Ok(lines) => lines,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("Makefile".to_string()),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    if includes == ["make/public.mk".to_string()] {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            &makefile,
            &ctx.repo_root,
            "Makefile must include exactly one file: make/public.mk",
        )])
    }
}

fn test_make_include_002_public_surface(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-INCLUDE-002";
    let test_id = "make.includes.public_surface";
    let public_path = ctx.repo_root.join("make/public.mk");
    let internal_path = ctx.repo_root.join("make/_internal.mk");
    let includes = match include_lines(&public_path) {
        Ok(lines) => lines,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&public_path, &ctx.repo_root)),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    let expected = vec![
        "make/_internal.mk".to_string(),
        "make/checks.mk".to_string(),
        "make/contracts.mk".to_string(),
        "make/macros.mk".to_string(),
        "make/paths.mk".to_string(),
        "make/vars.mk".to_string(),
    ];
    let internal_includes = match include_lines(&internal_path) {
        Ok(lines) => lines,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some(rel(&internal_path, &ctx.repo_root)),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    let mut violations = Vec::new();
    if includes != expected {
        violations.push(violation(
            contract_id,
            test_id,
            &public_path,
            &ctx.repo_root,
            "make/public.mk must include only vars, paths, macros, _internal, and checks",
        ));
    }
    if internal_includes != ["make/root.mk".to_string()] {
        violations.push(violation(
            contract_id,
            test_id,
            &internal_path,
            &ctx.repo_root,
            "make/_internal.mk must include exactly one file: make/root.mk",
        ));
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_include_003_acyclic(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-INCLUDE-003";
    let test_id = "make.includes.acyclic";
    let files = sorted_make_sources(&ctx.repo_root);
    let mut edges = BTreeMap::<String, Vec<String>>::new();
    let mut violations = Vec::new();
    for path in &files {
        let rel_path = rel(path, &ctx.repo_root);
        let includes = match include_lines(path) {
            Ok(lines) => lines,
            Err(err) => {
                violations.push(Violation {
                    contract_id: contract_id.to_string(),
                    test_id: test_id.to_string(),
                    file: Some(rel_path.clone()),
                    line: Some(1),
                    message: err,
                    evidence: None,
                });
                continue;
            }
        };
        edges.insert(rel_path, includes);
    }
    fn visit(
        node: &str,
        edges: &BTreeMap<String, Vec<String>>,
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

pub fn contracts(_repo_root: &Path) -> Result<Vec<Contract>, String> {
    let mut contracts = vec![
        Contract {
            id: ContractId("MAKE-DIR-001".to_string()),
            title: "make root docs boundary",
            tests: vec![TestCase {
                id: TestId("make.docs.allowed_root_docs_only".to_string()),
                title: "make root keeps only README.md and CONTRACT.md as markdown",
                kind: TestKind::Pure,
                run: test_make_dir_001_allowed_root_docs_only,
            }],
        },
        Contract {
            id: ContractId("MAKE-DIR-002".to_string()),
            title: "make nested docs removal",
            tests: vec![TestCase {
                id: TestId("make.docs.no_nested_markdown".to_string()),
                title: "make contains no nested markdown",
                kind: TestKind::Pure,
                run: test_make_dir_002_no_nested_markdown,
            }],
        },
        Contract {
            id: ContractId("MAKE-DIR-003".to_string()),
            title: "make root file boundary",
            tests: vec![TestCase {
                id: TestId("make.surface.allowed_root_files".to_string()),
                title: "make root contains only curated wrapper files",
                kind: TestKind::Pure,
                run: test_make_dir_003_allowed_root_files,
            }],
        },
        Contract {
            id: ContractId("MAKE-ENV-001".to_string()),
            title: "make env file singularity",
            tests: vec![TestCase {
                id: TestId("make.env.single_macros_and_runenv".to_string()),
                title: "make keeps one macros file and one run-environment file",
                kind: TestKind::Pure,
                run: test_make_env_001_single_macros_and_runenv,
            }],
        },
        Contract {
            id: ContractId("MAKE-ENV-002".to_string()),
            title: "make env role boundary",
            tests: vec![TestCase {
                id: TestId("make.env.role_boundary".to_string()),
                title: "macros and run-environment files keep distinct responsibilities",
                kind: TestKind::Pure,
                run: test_make_env_002_role_boundary,
            }],
        },
        Contract {
            id: ContractId("MAKE-INCLUDE-001".to_string()),
            title: "make root include entrypoint",
            tests: vec![TestCase {
                id: TestId("make.includes.root_single_entrypoint".to_string()),
                title: "Makefile includes only make/public.mk",
                kind: TestKind::Pure,
                run: test_make_include_001_root_single_entrypoint,
            }],
        },
        Contract {
            id: ContractId("MAKE-INCLUDE-002".to_string()),
            title: "make public include boundary",
            tests: vec![TestCase {
                id: TestId("make.includes.public_surface".to_string()),
                title: "make public entrypoint includes only the approved wrapper modules",
                kind: TestKind::Pure,
                run: test_make_include_002_public_surface,
            }],
        },
        Contract {
            id: ContractId("MAKE-INCLUDE-003".to_string()),
            title: "make include graph acyclic",
            tests: vec![TestCase {
                id: TestId("make.includes.acyclic".to_string()),
                title: "make include graph is acyclic",
                kind: TestKind::Pure,
                run: test_make_include_003_acyclic,
            }],
        },
        Contract {
            id: ContractId("MAKE-001".to_string()),
            title: "contracts gate uses make/contracts.mk as single entrypoint",
            tests: vec![TestCase {
                id: TestId("make.contracts.single_entrypoint".to_string()),
                title: "contracts targets are sourced from make/contracts.mk via make/public.mk",
                kind: TestKind::Pure,
                run: test_make_contracts_001_single_entrypoint,
            }],
        },
        Contract {
            id: ContractId("MAKE-002".to_string()),
            title: "contracts gate public targets are explicit and stable",
            tests: vec![TestCase {
                id: TestId("make.contracts.target_surface".to_string()),
                title: "contracts.mk declares only the approved contracts targets",
                kind: TestKind::Pure,
                run: test_make_contracts_002_target_surface,
            }],
        },
        Contract {
            id: ContractId("MAKE-003".to_string()),
            title: "contracts gate targets are thin delegates to the contracts runner",
            tests: vec![TestCase {
                id: TestId("make.contracts.delegate_only".to_string()),
                title: "contracts.mk delegates via bijux-dev-atlas contracts invocations only",
                kind: TestKind::Pure,
                run: test_make_contracts_003_delegate_only,
            }],
        },
    ];
    contracts.extend(surface_contracts::contracts());
    contracts.extend(wrapper_contracts::contracts());
    Ok(contracts)
}

pub fn contract_explain(contract_id: &str) -> String {
    match contract_id {
        "MAKE-DIR-001" => {
            "Keep make markdown authority limited to make/README.md and make/CONTRACT.md.".to_string()
        }
        "MAKE-DIR-002" => {
            "Remove nested markdown from make/ so implementation and policy do not drift."
                .to_string()
        }
        "MAKE-DIR-003" => {
            "Freeze the top-level make/ surface to curated wrapper files only.".to_string()
        }
        "MAKE-ENV-001" => {
            "Keep one macros source and one run-environment source to avoid env drift.".to_string()
        }
        "MAKE-ENV-002" => {
            "Separate pure macros from exported runtime defaults so responsibility stays obvious."
                .to_string()
        }
        "MAKE-INCLUDE-001" => {
            "Makefile must route through one public entrypoint instead of accumulating direct includes."
                .to_string()
        }
        "MAKE-INCLUDE-002" => {
            "The public make entrypoint must include only the approved wrapper modules.".to_string()
        }
        "MAKE-INCLUDE-003" => {
            "Keep the make include graph acyclic so wrapper composition stays reviewable.".to_string()
        }
        "MAKE-001" => "Define contracts gate targets in make/contracts.mk and include them through make/public.mk.".to_string(),
        "MAKE-002" => "Keep contracts public target surface explicit and stable so gate usage is predictable.".to_string(),
        "MAKE-003" => "Contracts make targets must remain thin delegates to bijux-dev-atlas contracts commands.".to_string(),
        _ => surface_contracts::contract_explain(contract_id)
            .or_else(|| wrapper_contracts::contract_explain(contract_id))
            .unwrap_or("Unknown make contract id.")
            .to_string(),
    }
}

pub fn contract_gate_command(_contract_id: &str) -> &'static str {
    "bijux dev atlas contracts make --mode static"
}
