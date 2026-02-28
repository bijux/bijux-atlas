// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use super::{
    rel, sorted_make_sources, Contract, ContractId, RunContext, TestCase, TestId, TestKind,
    TestResult, Violation,
};

fn failure(contract_id: &str, test_id: &str, file: &str, message: impl Into<String>) -> TestResult {
    TestResult::Fail(vec![Violation {
        contract_id: contract_id.to_string(),
        test_id: test_id.to_string(),
        file: Some(file.to_string()),
        line: Some(1),
        message: message.into(),
        evidence: None,
    }])
}

fn read_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
}

fn curated_targets(repo_root: &Path) -> Result<Vec<String>, String> {
    let text = read_text(&repo_root.join("make/makefiles/root.mk"))?;
    let mut inside = false;
    let mut targets = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !inside && trimmed.starts_with("CURATED_TARGETS :=") {
            inside = true;
        }
        if !inside {
            continue;
        }
        for token in trimmed
            .replace('\\', " ")
            .split_whitespace()
            .filter(|value| {
                value
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            })
        {
            targets.push(token.to_string());
        }
        if !trimmed.ends_with('\\') {
            break;
        }
    }
    targets.sort();
    targets.dedup();
    if targets.is_empty() {
        Err("CURATED_TARGETS is missing or empty".to_string())
    } else {
        Ok(targets)
    }
}

fn configured_public_targets(path: &Path) -> Result<(usize, Vec<String>), String> {
    let json: Value = serde_json::from_str(&read_text(path)?)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let max = json
        .get("max_public_targets")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{} missing max_public_targets", path.display()))?
        as usize;
    let mut names = json
        .get("public_targets")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing public_targets array", path.display()))?
        .iter()
        .filter_map(|value| value.get("name").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok((max, names))
}

fn target_list_targets(path: &Path) -> Result<Vec<String>, String> {
    let json: Value = serde_json::from_str(&read_text(path)?)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let mut names = json
        .get("public_targets")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing public_targets array", path.display()))?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok(names)
}

fn declared_targets(repo_root: &Path) -> Result<BTreeMap<String, (String, Vec<String>)>, String> {
    let mut targets = BTreeMap::<String, (String, Vec<String>)>::new();
    for path in sorted_make_sources(repo_root) {
        let file = rel(&path, repo_root);
        let text = read_text(&path)?;
        let mut current = None::<String>;
        for line in text.lines() {
            if line.starts_with('\t') {
                if let Some(name) = current.as_ref() {
                    if let Some((_, recipes)) = targets.get_mut(name) {
                        recipes.push(line.trim().to_string());
                    }
                }
                continue;
            }
            current = None;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('.') {
                continue;
            }
            if trimmed.contains(":=") || trimmed.contains("?=") {
                continue;
            }
            let Some((head, _)) = trimmed.split_once(':') else {
                continue;
            };
            let name = head.trim();
            if !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                continue;
            }
            let name = name.to_string();
            targets
                .entry(name.clone())
                .or_insert((file.clone(), Vec::new()));
            current = Some(name);
        }
    }
    Ok(targets)
}

fn is_auxiliary_recipe(recipe: &str) -> bool {
    let trimmed = recipe.trim_start_matches('@').trim();
    trimmed.starts_with("printf ")
        || trimmed.starts_with("mkdir -p ")
        || trimmed.starts_with("targets=")
        || trimmed.starts_with("output=")
        || trimmed.starts_with("status=")
        || trimmed.starts_with("if ")
        || trimmed.starts_with("fi")
        || trimmed.starts_with("for ")
        || trimmed.starts_with("test ")
        || trimmed.starts_with("exit ")
        || trimmed.starts_with("rm -f ")
}

fn test_make_surface_001_single_source(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("make/target-list.json");
    match read_text(&path) {
        Ok(text) if text.contains("\"source\": \"make/makefiles/root.mk:CURATED_TARGETS\"") => {
            TestResult::Pass
        }
        Ok(_) => failure("MAKE-SURFACE-001", "make.surface.single_source", "make/target-list.json", "make/target-list.json must declare make/makefiles/root.mk:CURATED_TARGETS as its single source"),
        Err(err) => failure("MAKE-SURFACE-001", "make.surface.single_source", "make/target-list.json", err),
    }
}

fn test_make_surface_002_count_budget(ctx: &RunContext) -> TestResult {
    let targets = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return failure(
                "MAKE-SURFACE-002",
                "make.surface.count_budget",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let (max, _) =
        match configured_public_targets(&ctx.repo_root.join("configs/make/public-targets.json")) {
            Ok(values) => values,
            Err(err) => {
                return failure(
                    "MAKE-SURFACE-002",
                    "make.surface.count_budget",
                    "configs/make/public-targets.json",
                    err,
                )
            }
        };
    if targets.len() <= max {
        TestResult::Pass
    } else {
        failure(
            "MAKE-SURFACE-002",
            "make.surface.count_budget",
            "make/makefiles/root.mk",
            format!(
                "curated target count {} exceeds max_public_targets {}",
                targets.len(),
                max
            ),
        )
    }
}

fn test_make_surface_003_registry_sync(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return failure(
                "MAKE-SURFACE-003",
                "make.surface.registry_sync",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let listed = match target_list_targets(&ctx.repo_root.join("make/target-list.json")) {
        Ok(targets) => targets,
        Err(err) => {
            return failure(
                "MAKE-SURFACE-003",
                "make.surface.registry_sync",
                "make/target-list.json",
                err,
            )
        }
    };
    let (_, configured) =
        match configured_public_targets(&ctx.repo_root.join("configs/make/public-targets.json")) {
            Ok(values) => values,
            Err(err) => {
                return failure(
                    "MAKE-SURFACE-003",
                    "make.surface.registry_sync",
                    "configs/make/public-targets.json",
                    err,
                )
            }
        };
    if curated == listed && curated == configured {
        TestResult::Pass
    } else {
        failure("MAKE-SURFACE-003", "make.surface.registry_sync", "make", "curated targets, make/target-list.json, and configs/make/public-targets.json must match exactly")
    }
}

fn test_make_surface_005_delegate_only(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return failure(
                "MAKE-SURFACE-005",
                "make.surface.delegate_only",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return failure(
                "MAKE-SURFACE-005",
                "make.surface.delegate_only",
                "make",
                err,
            )
        }
    };
    for target in curated {
        let Some((file, recipes)) = declared.get(&target) else {
            return failure(
                "MAKE-SURFACE-005",
                "make.surface.delegate_only",
                "make",
                format!("curated target {target} is not declared"),
            );
        };
        if recipes.is_empty() {
            continue;
        }
        if ["help", "make-target-list"].contains(&target.as_str())
            && recipes
                .iter()
                .all(|recipe| is_auxiliary_recipe(recipe) || recipe.contains("python3 "))
        {
            continue;
        }
        let has_delegate = recipes.iter().any(|recipe| {
            recipe.contains("$(DEV_ATLAS)")
                || recipe.contains("cargo ")
                || recipe.contains("$(MAKE)")
        });
        if has_delegate {
            continue;
        }
        return failure(
            "MAKE-SURFACE-005",
            "make.surface.delegate_only",
            file,
            format!(
                "curated target {target} must delegate through $(DEV_ATLAS), cargo, or $(MAKE)"
            ),
        );
    }
    TestResult::Pass
}

fn test_make_internal_001_root_helpers_prefixed(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets.into_iter().collect::<BTreeSet<_>>(),
        Err(err) => {
            return failure(
                "MAKE-INTERNAL-001",
                "make.internal.root_helpers_prefixed",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return failure(
                "MAKE-INTERNAL-001",
                "make.internal.root_helpers_prefixed",
                "make",
                err,
            )
        }
    };
    for (name, (file, _)) in declared {
        if file == "make/makefiles/root.mk"
            && !curated.contains(&name)
            && !name.starts_with("_internal-")
        {
            return failure(
                "MAKE-INTERNAL-001",
                "make.internal.root_helpers_prefixed",
                &file,
                format!("non-curated root target {name} must use the _internal- prefix"),
            );
        }
    }
    TestResult::Pass
}

fn test_sources_forbid(
    ctx: &RunContext,
    contract_id: &str,
    test_id: &str,
    message: &str,
    patterns: &[&str],
) -> TestResult {
    for path in sorted_make_sources(&ctx.repo_root) {
        let text = match read_text(&path) {
            Ok(text) => text,
            Err(err) => return failure(contract_id, test_id, &rel(&path, &ctx.repo_root), err),
        };
        if patterns.iter().any(|pattern| text.contains(pattern)) {
            return failure(contract_id, test_id, &rel(&path, &ctx.repo_root), message);
        }
    }
    TestResult::Pass
}

fn test_make_repro_001_runenv_exports(ctx: &RunContext) -> TestResult {
    match read_text(&ctx.repo_root.join("make/makefiles/_runenv.mk")) {
        Ok(text)
            if text.contains("export ")
                && text.contains("RUN_ID")
                && text.contains("CARGO_TARGET_DIR") =>
        {
            TestResult::Pass
        }
        Ok(_) => failure(
            "MAKE-REPRO-001",
            "make.repro.runenv_exports",
            "make/makefiles/_runenv.mk",
            "run environment file must export deterministic run and tool cache variables",
        ),
        Err(err) => failure(
            "MAKE-REPRO-001",
            "make.repro.runenv_exports",
            "make/makefiles/_runenv.mk",
            err,
        ),
    }
}

fn test_make_ci_001_curated_workflow_usage(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets.into_iter().collect::<BTreeSet<_>>(),
        Err(err) => {
            return failure(
                "MAKE-CI-001",
                "make.ci.curated_workflow_usage",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(ctx.repo_root.join(".github/workflows")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("yml" | "yaml")
            ) {
                files.push(path);
            }
        }
    }
    files.sort();
    for path in files {
        let text = match read_text(&path) {
            Ok(text) => text,
            Err(err) => {
                return failure(
                    "MAKE-CI-001",
                    "make.ci.curated_workflow_usage",
                    &rel(&path, &ctx.repo_root),
                    err,
                )
            }
        };
        let tokens = text.split_whitespace().collect::<Vec<_>>();
        for pair in tokens.windows(2) {
            if pair[0] == "make"
                && pair[1]
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                && !curated.contains(pair[1])
            {
                return failure(
                    "MAKE-CI-001",
                    "make.ci.curated_workflow_usage",
                    &rel(&path, &ctx.repo_root),
                    format!("workflow uses non-curated make target {}", pair[1]),
                );
            }
        }
    }
    TestResult::Pass
}

fn test_make_struct_002_mk_only(ctx: &RunContext) -> TestResult {
    let Ok(entries) = std::fs::read_dir(ctx.repo_root.join("make/makefiles")) else {
        return failure(
            "MAKE-STRUCT-002",
            "make.structure.mk_only",
            "make/makefiles",
            "make/makefiles directory is missing",
        );
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|value| value.to_str()) != Some("mk") {
            return failure(
                "MAKE-STRUCT-002",
                "make.structure.mk_only",
                &rel(&path, &ctx.repo_root),
                "make/makefiles may contain only .mk files",
            );
        }
    }
    TestResult::Pass
}

fn test_make_ops_001_ops_targets_use_control_plane(ctx: &RunContext) -> TestResult {
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => return failure("MAKE-OPS-001", "make.ops.control_plane_only", "make", err),
    };
    for (name, (file, recipes)) in declared {
        if !(name.starts_with("ops-") || name.starts_with("k8s-") || name.starts_with("stack-"))
            || recipes.is_empty()
        {
            continue;
        }
        if recipes.iter().all(|recipe| {
            recipe.contains("$(DEV_ATLAS) ops")
                || recipe.contains("$(DEV_ATLAS) contracts ops")
                || recipe.contains("$(DEV_ATLAS) check")
                || recipe.contains("$(MAKE)")
                || is_auxiliary_recipe(recipe)
        }) {
            continue;
        }
        return failure(
            "MAKE-OPS-001",
            "make.ops.control_plane_only",
            &file,
            format!("target {name} must delegate through $(DEV_ATLAS) ops"),
        );
    }
    TestResult::Pass
}

fn test_make_docker_001_docker_targets_use_contract_runner(ctx: &RunContext) -> TestResult {
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return failure(
                "MAKE-DOCKER-001",
                "make.docker.contract_runner_only",
                "make",
                err,
            )
        }
    };
    for name in ["docker-contracts", "docker-contracts-effect", "docker-gate"] {
        let Some((file, recipes)) = declared.get(name) else {
            return failure(
                "MAKE-DOCKER-001",
                "make.docker.contract_runner_only",
                "make/makefiles/docker.mk",
                format!("missing target {name}"),
            );
        };
        if recipes.iter().all(|recipe| {
            recipe.contains("$(DEV_ATLAS) contracts docker")
                || recipe.contains("$(MAKE)")
                || is_auxiliary_recipe(recipe)
        }) {
            continue;
        }
        return failure(
            "MAKE-DOCKER-001",
            "make.docker.contract_runner_only",
            file,
            format!("target {name} must delegate through $(DEV_ATLAS) contracts docker"),
        );
    }
    TestResult::Pass
}

fn test_make_ssot_001_checks_delegate_to_contracts(ctx: &RunContext) -> TestResult {
    match read_text(&ctx.repo_root.join("make/checks.mk")) {
        Ok(text) if text.contains("$(DEV_ATLAS) contracts make --mode static") && !text.contains("rg -n") => TestResult::Pass,
        Ok(_) => failure("MAKE-SSOT-001", "make.ssot.checks_delegate_to_contracts", "make/checks.mk", "make/checks.mk must delegate contract authority to `bijux dev atlas contracts make` and avoid embedded grep logic"),
        Err(err) => failure("MAKE-SSOT-001", "make.ssot.checks_delegate_to_contracts", "make/checks.mk", err),
    }
}

pub(super) fn contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("MAKE-SURFACE-001".to_string()),
            title: "make curated source of truth",
            tests: vec![TestCase {
                id: TestId("make.surface.single_source".to_string()),
                title: "make target list declares root curated targets as its only source",
                kind: TestKind::Pure,
                run: test_make_surface_001_single_source,
            }],
        },
        Contract {
            id: ContractId("MAKE-SURFACE-002".to_string()),
            title: "make curated target budget",
            tests: vec![TestCase {
                id: TestId("make.surface.count_budget".to_string()),
                title: "curated target count stays within the configured max",
                kind: TestKind::Pure,
                run: test_make_surface_002_count_budget,
            }],
        },
        Contract {
            id: ContractId("MAKE-SURFACE-003".to_string()),
            title: "make curated registry sync",
            tests: vec![TestCase {
                id: TestId("make.surface.registry_sync".to_string()),
                title: "curated targets, config registry, and target list stay in sync",
                kind: TestKind::Pure,
                run: test_make_surface_003_registry_sync,
            }],
        },
        Contract {
            id: ContractId("MAKE-SURFACE-005".to_string()),
            title: "make delegate only wrappers",
            tests: vec![TestCase {
                id: TestId("make.surface.delegate_only".to_string()),
                title: "curated targets stay thin and delegate through cargo, make, or dev atlas",
                kind: TestKind::Pure,
                run: test_make_surface_005_delegate_only,
            }],
        },
        Contract {
            id: ContractId("MAKE-INTERNAL-001".to_string()),
            title: "make internal target prefix",
            tests: vec![TestCase {
                id: TestId("make.internal.root_helpers_prefixed".to_string()),
                title: "non-curated root helpers use the _internal- prefix",
                kind: TestKind::Pure,
                run: test_make_internal_001_root_helpers_prefixed,
            }],
        },
        Contract {
            id: ContractId("MAKE-INTERNAL-002".to_string()),
            title: "make scripts banned",
            tests: vec![TestCase {
                id: TestId("make.internal.no_scripts_path".to_string()),
                title: "make sources do not invoke scripts/ directly",
                kind: TestKind::Pure,
                run: |ctx| {
                    test_sources_forbid(
                        ctx,
                        "MAKE-INTERNAL-002",
                        "make.internal.no_scripts_path",
                        "make sources must not invoke scripts/ directly",
                        &["scripts/"],
                    )
                },
            }],
        },
        Contract {
            id: ContractId("MAKE-NET-001".to_string()),
            title: "make network commands banned",
            tests: vec![TestCase {
                id: TestId("make.network.no_curl_or_wget".to_string()),
                title: "make sources do not call curl or wget",
                kind: TestKind::Pure,
                run: |ctx| {
                    test_sources_forbid(
                        ctx,
                        "MAKE-NET-001",
                        "make.network.no_curl_or_wget",
                        "make sources must not call curl or wget",
                        &["curl", "wget"],
                    )
                },
            }],
        },
        Contract {
            id: ContractId("MAKE-SHELL-001".to_string()),
            title: "make shell path stability",
            tests: vec![TestCase {
                id: TestId("make.shell.no_cd".to_string()),
                title: "make sources do not depend on cd chains",
                kind: TestKind::Pure,
                run: |ctx| {
                    test_sources_forbid(
                        ctx,
                        "MAKE-SHELL-001",
                        "make.shell.no_cd",
                        "make sources must not use cd in recipes",
                        &["\tcd "],
                    )
                },
            }],
        },
        Contract {
            id: ContractId("MAKE-REPRO-001".to_string()),
            title: "make runenv exports",
            tests: vec![TestCase {
                id: TestId("make.repro.runenv_exports".to_string()),
                title: "run environment exports deterministic run and cache variables",
                kind: TestKind::Pure,
                run: test_make_repro_001_runenv_exports,
            }],
        },
        Contract {
            id: ContractId("MAKE-CI-001".to_string()),
            title: "make workflow curated usage",
            tests: vec![TestCase {
                id: TestId("make.ci.curated_workflow_usage".to_string()),
                title: "workflows call only curated public make targets",
                kind: TestKind::Pure,
                run: test_make_ci_001_curated_workflow_usage,
            }],
        },
        Contract {
            id: ContractId("MAKE-STRUCT-002".to_string()),
            title: "make wrapper files only",
            tests: vec![TestCase {
                id: TestId("make.structure.mk_only".to_string()),
                title: "make/makefiles contains only .mk files",
                kind: TestKind::Pure,
                run: test_make_struct_002_mk_only,
            }],
        },
        Contract {
            id: ContractId("MAKE-OPS-001".to_string()),
            title: "make ops control plane boundary",
            tests: vec![TestCase {
                id: TestId("make.ops.control_plane_only".to_string()),
                title: "ops, k8s, and stack targets route through the ops control plane",
                kind: TestKind::Pure,
                run: test_make_ops_001_ops_targets_use_control_plane,
            }],
        },
        Contract {
            id: ContractId("MAKE-DOCKER-001".to_string()),
            title: "make docker contract boundary",
            tests: vec![TestCase {
                id: TestId("make.docker.contract_runner_only".to_string()),
                title: "docker contract targets route through the docker contracts runner",
                kind: TestKind::Pure,
                run: test_make_docker_001_docker_targets_use_contract_runner,
            }],
        },
        Contract {
            id: ContractId("MAKE-DRIFT-001".to_string()),
            title: "make target list drift",
            tests: vec![TestCase {
                id: TestId("make.surface.target_list_drift".to_string()),
                title: "make target list artifact matches the curated target source",
                kind: TestKind::Pure,
                run: test_make_surface_003_registry_sync,
            }],
        },
        Contract {
            id: ContractId("MAKE-SSOT-001".to_string()),
            title: "make contracts authority",
            tests: vec![TestCase {
                id: TestId("make.ssot.checks_delegate_to_contracts".to_string()),
                title: "make checks delegate contract authority to the Rust contracts runner",
                kind: TestKind::Pure,
                run: test_make_ssot_001_checks_delegate_to_contracts,
            }],
        },
    ]
}

pub(super) fn contract_explain(contract_id: &str) -> Option<&'static str> {
    match contract_id {
        "MAKE-SURFACE-001" => Some("Use make/makefiles/root.mk:CURATED_TARGETS as the single curated target source."),
        "MAKE-SURFACE-002" => Some("Keep the public make surface within the explicit target budget."),
        "MAKE-SURFACE-003" => Some("Keep the curated target source, target artifact, and config registry synchronized."),
        "MAKE-SURFACE-005" => Some("Curated make targets must stay thin delegates instead of becoming an execution engine."),
        "MAKE-INTERNAL-001" => Some("Non-curated helpers in the root wrapper layer must remain visibly internal."),
        "MAKE-INTERNAL-002" => Some("Make wrappers must not bypass the control plane through direct scripts."),
        "MAKE-NET-001" => Some("Make wrappers must not own network fetches."),
        "MAKE-SHELL-001" => Some("Make wrappers must not depend on directory-changing shell chains."),
        "MAKE-REPRO-001" => Some("Public wrappers must route through exported deterministic run environment defaults."),
        "MAKE-CI-001" => Some("CI workflows may call only the curated make surface."),
        "MAKE-STRUCT-002" => Some("The make/makefiles tree must contain only executable wrapper modules."),
        "MAKE-OPS-001" => Some("Ops-related make targets must route through `bijux dev atlas ops ...`."),
        "MAKE-DOCKER-001" => Some("Docker-related make targets must route through the docker contract runner."),
        "MAKE-DRIFT-001" => Some("The committed target list artifact must stay aligned with the curated target source."),
        "MAKE-SSOT-001" => Some("Rust contracts, not Make grep rules, are the authority for make contract enforcement."),
        _ => None,
    }
}
