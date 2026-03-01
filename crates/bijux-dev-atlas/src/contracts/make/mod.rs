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

fn top_level_make_module_files(repo_root: &Path) -> Vec<PathBuf> {
    top_level_make_files(repo_root)
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("mk"))
        .collect()
}

fn curated_targets(repo_root: &Path) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(repo_root.join("make/root.mk"))
        .map_err(|err| format!("read make/root.mk failed: {err}"))?;
    let mut collecting = false;
    let mut targets = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !collecting && trimmed.starts_with("CURATED_TARGETS :=") {
            collecting = true;
        }
        if !collecting {
            continue;
        }
        for token in trimmed.replace('\\', " ").split_whitespace() {
            if token
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            {
                targets.push(token.to_string());
            }
        }
        if !trimmed.ends_with('\\') {
            break;
        }
    }
    if targets.is_empty() {
        Err("CURATED_TARGETS is missing or empty".to_string())
    } else {
        Ok(targets)
    }
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

fn test_make_struct_001_root_makefile_minimal(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-STRUCT-001";
    let test_id = "make.structure.root_makefile_minimal";
    let makefile = ctx.repo_root.join("Makefile");
    let text = match std::fs::read_to_string(&makefile) {
        Ok(text) => text,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("Makefile".to_string()),
                line: Some(1),
                message: format!("read Makefile failed: {err}"),
                evidence: None,
            }]);
        }
    };
    let line_count = text.lines().count();
    let non_empty = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if line_count <= 200 && non_empty == vec!["include make/public.mk"] {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            contract_id,
            test_id,
            &makefile,
            &ctx.repo_root,
            "Makefile must stay <=200 lines and contain only `include make/public.mk`",
        )])
    }
}

fn test_make_struct_003_allowed_modules(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-STRUCT-003";
    let test_id = "make.structure.allowed_modules";
    let allowed = BTreeSet::from([
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
        "vars.mk",
        "verification.mk",
    ]);
    let mut violations = Vec::new();
    for path in top_level_make_module_files(&ctx.repo_root) {
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !allowed.contains(name) {
            violations.push(violation(
                contract_id,
                test_id,
                &path,
                &ctx.repo_root,
                "make module file is outside the approved top-level whitelist",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_struct_004_module_headers(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-STRUCT-004";
    let test_id = "make.structure.module_headers";
    let mut violations = Vec::new();
    for path in top_level_make_module_files(&ctx.repo_root) {
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                violations.push(violation(
                    contract_id,
                    test_id,
                    &path,
                    &ctx.repo_root,
                    &format!("read failed: {err}"),
                ));
                continue;
            }
        };
        let lines = text.lines().take(3).collect::<Vec<_>>();
        let has_scope = lines.iter().any(|line| line.starts_with("# Scope:"));
        let has_public_targets = lines
            .iter()
            .any(|line| line.starts_with("# Public targets:"));
        if !has_scope || !has_public_targets {
            violations.push(violation(
                contract_id,
                test_id,
                &path,
                &ctx.repo_root,
                "make module must start with `# Scope:` and `# Public targets:` header lines",
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_make_struct_007_help_surface_sorted(ctx: &RunContext) -> TestResult {
    let contract_id = "MAKE-STRUCT-007";
    let test_id = "make.structure.help_surface_sorted";
    let targets = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return TestResult::Fail(vec![Violation {
                contract_id: contract_id.to_string(),
                test_id: test_id.to_string(),
                file: Some("make/root.mk".to_string()),
                line: Some(1),
                message: err,
                evidence: None,
            }]);
        }
    };
    let mut sorted = targets.clone();
    sorted.sort();
    sorted.dedup();
    if targets == sorted {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![Violation {
            contract_id: contract_id.to_string(),
            test_id: test_id.to_string(),
            file: Some("make/root.mk".to_string()),
            line: Some(1),
            message: "CURATED_TARGETS must stay alphabetically sorted and duplicate-free so `make help` output is stable".to_string(),
            evidence: None,
        }])
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


include!("mod_include_and_catalog.inc.rs");
