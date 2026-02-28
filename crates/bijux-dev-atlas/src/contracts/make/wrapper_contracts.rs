// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use super::{
    rel, sorted_make_sources, Contract, ContractId, RunContext, TestCase, TestId, TestKind,
    TestResult, Violation,
};

fn fail(contract_id: &str, test_id: &str, file: &str, message: impl Into<String>) -> TestResult {
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

fn curated_targets(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let text = read_text(&repo_root.join("make/makefiles/root.mk"))?;
    let mut inside = false;
    let mut targets = BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !inside && trimmed.starts_with("CURATED_TARGETS :=") {
            inside = true;
        }
        if !inside {
            continue;
        }
        for token in trimmed.replace('\\', " ").split_whitespace() {
            if token
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            {
                targets.insert(token.to_string());
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
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with('.')
                || trimmed.contains(":=")
                || trimmed.contains("?=")
            {
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

fn count_recipe_pipes(recipe: &str) -> usize {
    recipe.match_indices(" | ").count()
}

fn file_line_count(path: &Path) -> Result<usize, String> {
    read_text(path).map(|text| text.lines().count())
}

fn line_invokes_direct_tool(line: &str) -> bool {
    let trimmed = line.trim_start();
    let trimmed = trimmed.strip_prefix('@').unwrap_or(trimmed).trim_start();
    ["kubectl ", "helm ", "docker ", "k6 "]
        .iter()
        .any(|pattern| trimmed.starts_with(pattern))
}

fn make_registry_targets(
    repo_root: &Path,
) -> Result<BTreeMap<String, (String, Vec<String>)>, String> {
    let path = repo_root.join("configs/ops/make-target-registry.json");
    let json: Value = serde_json::from_str(&read_text(&path)?)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let mut rows = BTreeMap::new();
    for row in json
        .get("targets")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{} missing targets array", path.display()))?
    {
        let Some(name) = row.get("name").and_then(Value::as_str) else {
            continue;
        };
        let visibility = row
            .get("visibility")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let defined_in = row
            .get("defined_in")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        rows.insert(name.to_string(), (visibility, defined_in));
    }
    Ok(rows)
}

fn test_make_targetlist_001_explicit_policy(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("make/target-list.json");
    let text = match read_text(&path) {
        Ok(text) => text,
        Err(err) => {
            return fail(
                "MAKE-TARGETLIST-001",
                "make.target_list.explicit_policy",
                "make/target-list.json",
                err,
            )
        }
    };
    let json: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            return fail(
                "MAKE-TARGETLIST-001",
                "make.target_list.explicit_policy",
                "make/target-list.json",
                format!("parse make/target-list.json failed: {err}"),
            )
        }
    };
    if json.get("schema_version").and_then(Value::as_u64) == Some(1)
        && json.get("source").and_then(Value::as_str)
            == Some("make/makefiles/root.mk:CURATED_TARGETS")
    {
        TestResult::Pass
    } else {
        fail("MAKE-TARGETLIST-001", "make.target_list.explicit_policy", "make/target-list.json", "make/target-list.json must be a committed schema_version=1 registry sourced from make/makefiles/root.mk:CURATED_TARGETS")
    }
}

fn test_make_name_001_helper_files_prefixed(ctx: &RunContext) -> TestResult {
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-NAME-001",
                "make.naming.helper_files_prefixed",
                "make/makefiles",
                err,
            )
        }
    };
    let mut files_with_targets = BTreeSet::new();
    for (_, (file, _)) in declared {
        if file.starts_with("make/makefiles/") {
            files_with_targets.insert(file);
        }
    }
    let root = ctx.repo_root.join("make/makefiles");
    let Ok(entries) = std::fs::read_dir(root) else {
        return fail(
            "MAKE-NAME-001",
            "make.naming.helper_files_prefixed",
            "make/makefiles",
            "make/makefiles directory is missing",
        );
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file = rel(&path, &ctx.repo_root);
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !files_with_targets.contains(&file) && !name.starts_with('_') {
            return fail(
                "MAKE-NAME-001",
                "make.naming.helper_files_prefixed",
                &file,
                "helper-only makefile includes must use the _*.mk naming pattern",
            );
        }
    }
    TestResult::Pass
}

fn test_make_name_002_public_files_not_prefixed(ctx: &RunContext) -> TestResult {
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-NAME-002",
                "make.naming.public_files_clear",
                "make/makefiles",
                err,
            )
        }
    };
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-NAME-002",
                "make.naming.public_files_clear",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let mut files = BTreeSet::new();
    for (target, (file, _)) in declared {
        if !curated.contains(&target) {
            continue;
        }
        files.insert(file);
    }
    for file in files {
        let name = Path::new(&file)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if name.starts_with('_') {
            return fail(
                "MAKE-NAME-002",
                "make.naming.public_files_clear",
                &file,
                "files that declare curated public targets must not start with _",
            );
        }
    }
    TestResult::Pass
}

fn test_make_art_001_run_scoped_artifacts(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-ART-001",
                "make.artifacts.run_scoped",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => return fail("MAKE-ART-001", "make.artifacts.run_scoped", "make", err),
    };
    for (target, (file, recipes)) in declared {
        if !curated.contains(&target) {
            continue;
        }
        for recipe in recipes {
            if recipe.contains("$(ARTIFACT_ROOT)/") && !recipe.contains("$(RUN_ID)") {
                return fail(
                    "MAKE-ART-001",
                    "make.artifacts.run_scoped",
                    &file,
                    format!("target {target} must keep artifact paths run-scoped with $(RUN_ID)"),
                );
            }
        }
    }
    TestResult::Pass
}

fn test_make_shell_002_no_multi_hop_pipes(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-SHELL-002",
                "make.shell.no_multi_hop_pipes",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let declared = match declared_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-SHELL-002",
                "make.shell.no_multi_hop_pipes",
                "make",
                err,
            )
        }
    };
    for (target, (file, recipes)) in declared {
        if !curated.contains(&target) {
            continue;
        }
        if target == "make-target-list" {
            continue;
        }
        if recipes.iter().any(|recipe| count_recipe_pipes(recipe) > 1) {
            return fail(
                "MAKE-SHELL-002",
                "make.shell.no_multi_hop_pipes",
                &file,
                format!("target {target} must not use multi-hop shell pipelines"),
            );
        }
    }
    TestResult::Pass
}

fn test_make_engine_001_no_direct_tool_invocation(ctx: &RunContext) -> TestResult {
    for path in sorted_make_sources(&ctx.repo_root) {
        let text = match read_text(&path) {
            Ok(text) => text,
            Err(err) => {
                return fail(
                    "MAKE-ENGINE-001",
                    "make.engine.no_direct_tools",
                    &rel(&path, &ctx.repo_root),
                    err,
                )
            }
        };
        if text.lines().any(line_invokes_direct_tool) {
            return fail(
                "MAKE-ENGINE-001",
                "make.engine.no_direct_tools",
                &rel(&path, &ctx.repo_root),
                "make wrappers must not invoke kubectl, helm, docker, or k6 directly",
            );
        }
    }
    TestResult::Pass
}

fn test_make_docs_001_line_budgets(ctx: &RunContext) -> TestResult {
    let readme_path = ctx.repo_root.join("make/README.md");
    let readme_lines = match file_line_count(&readme_path) {
        Ok(lines) => lines,
        Err(err) => {
            return fail(
                "MAKE-DOCS-001",
                "make.docs.line_budgets",
                "make/README.md",
                err,
            )
        }
    };
    if readme_lines > 120 {
        return fail(
            "MAKE-DOCS-001",
            "make.docs.line_budgets",
            "make/README.md",
            format!("make/README.md exceeds the 120-line budget ({readme_lines} lines)"),
        );
    }

    let contract_path = ctx.repo_root.join("make/CONTRACT.md");
    let contract_lines = match file_line_count(&contract_path) {
        Ok(lines) => lines,
        Err(err) => {
            return fail(
                "MAKE-DOCS-001",
                "make.docs.line_budgets",
                "make/CONTRACT.md",
                err,
            )
        }
    };
    if contract_lines > 200 {
        return fail(
            "MAKE-DOCS-001",
            "make.docs.line_budgets",
            "make/CONTRACT.md",
            format!("make/CONTRACT.md exceeds the 200-line budget ({contract_lines} lines)"),
        );
    }

    TestResult::Pass
}

fn test_make_gates_001_curated_targets_mapped(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-GATES-001",
                "make.gates.curated_targets_mapped",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let registry = match make_registry_targets(&ctx.repo_root) {
        Ok(rows) => rows,
        Err(err) => {
            return fail(
                "MAKE-GATES-001",
                "make.gates.curated_targets_mapped",
                "configs/ops/make-target-registry.json",
                err,
            )
        }
    };
    for target in curated {
        if !registry.contains_key(&target) {
            return fail(
                "MAKE-GATES-001",
                "make.gates.curated_targets_mapped",
                "configs/ops/make-target-registry.json",
                format!("curated target {target} is missing from the make target registry"),
            );
        }
    }
    TestResult::Pass
}

fn test_make_gates_002_curated_targets_public(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-GATES-002",
                "make.gates.curated_targets_public",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let registry = match make_registry_targets(&ctx.repo_root) {
        Ok(rows) => rows,
        Err(err) => {
            return fail(
                "MAKE-GATES-002",
                "make.gates.curated_targets_public",
                "configs/ops/make-target-registry.json",
                err,
            )
        }
    };
    for target in curated {
        let Some((visibility, _)) = registry.get(&target) else {
            continue;
        };
        if visibility != "public" {
            return fail(
                "MAKE-GATES-002",
                "make.gates.curated_targets_public",
                "configs/ops/make-target-registry.json",
                format!(
                    "curated target {target} must be marked public in the make target registry"
                ),
            );
        }
    }
    TestResult::Pass
}

fn test_make_gates_003_no_orphan_public_targets(ctx: &RunContext) -> TestResult {
    let curated = match curated_targets(&ctx.repo_root) {
        Ok(targets) => targets,
        Err(err) => {
            return fail(
                "MAKE-GATES-003",
                "make.gates.no_orphan_public_targets",
                "make/makefiles/root.mk",
                err,
            )
        }
    };
    let registry = match make_registry_targets(&ctx.repo_root) {
        Ok(rows) => rows,
        Err(err) => {
            return fail(
                "MAKE-GATES-003",
                "make.gates.no_orphan_public_targets",
                "configs/ops/make-target-registry.json",
                err,
            )
        }
    };
    for (name, (visibility, _)) in registry {
        if visibility == "public" && !curated.contains(&name) {
            return fail(
                "MAKE-GATES-003",
                "make.gates.no_orphan_public_targets",
                "configs/ops/make-target-registry.json",
                format!("public registry target {name} is not part of CURATED_TARGETS"),
            );
        }
    }
    TestResult::Pass
}

pub(super) fn contracts() -> Vec<Contract> {
    vec![
        Contract {
            id: ContractId("MAKE-TARGETLIST-001".to_string()),
            title: "make target list policy",
            tests: vec![TestCase {
                id: TestId("make.target_list.explicit_policy".to_string()),
                title: "make target list keeps an explicit committed policy header",
                kind: TestKind::Pure,
                run: test_make_targetlist_001_explicit_policy,
            }],
        },
        Contract {
            id: ContractId("MAKE-NAME-001".to_string()),
            title: "make helper file naming",
            tests: vec![TestCase {
                id: TestId("make.naming.helper_files_prefixed".to_string()),
                title: "helper-only makefile includes use the _*.mk naming pattern",
                kind: TestKind::Pure,
                run: test_make_name_001_helper_files_prefixed,
            }],
        },
        Contract {
            id: ContractId("MAKE-NAME-002".to_string()),
            title: "make public file naming",
            tests: vec![TestCase {
                id: TestId("make.naming.public_files_clear".to_string()),
                title: "files that declare curated public targets do not use helper prefixes",
                kind: TestKind::Pure,
                run: test_make_name_002_public_files_not_prefixed,
            }],
        },
        Contract {
            id: ContractId("MAKE-ART-001".to_string()),
            title: "make run scoped artifacts",
            tests: vec![TestCase {
                id: TestId("make.artifacts.run_scoped".to_string()),
                title: "curated targets keep artifact writes under run-scoped paths",
                kind: TestKind::Pure,
                run: test_make_art_001_run_scoped_artifacts,
            }],
        },
        Contract {
            id: ContractId("MAKE-SHELL-002".to_string()),
            title: "make shell pipeline bound",
            tests: vec![TestCase {
                id: TestId("make.shell.no_multi_hop_pipes".to_string()),
                title: "curated targets avoid multi-hop shell pipelines",
                kind: TestKind::Pure,
                run: test_make_shell_002_no_multi_hop_pipes,
            }],
        },
        Contract {
            id: ContractId("MAKE-ENGINE-001".to_string()),
            title: "make direct tool boundary",
            tests: vec![TestCase {
                id: TestId("make.engine.no_direct_tools".to_string()),
                title: "make wrappers do not invoke infra tools directly",
                kind: TestKind::Pure,
                run: test_make_engine_001_no_direct_tool_invocation,
            }],
        },
        Contract {
            id: ContractId("MAKE-DOCS-001".to_string()),
            title: "make docs line budgets",
            tests: vec![TestCase {
                id: TestId("make.docs.line_budgets".to_string()),
                title: "make docs stay inside bounded review budgets",
                kind: TestKind::Pure,
                run: test_make_docs_001_line_budgets,
            }],
        },
        Contract {
            id: ContractId("MAKE-GATES-001".to_string()),
            title: "make gate mapping coverage",
            tests: vec![TestCase {
                id: TestId("make.gates.curated_targets_mapped".to_string()),
                title: "every curated target appears in the make target registry",
                kind: TestKind::Pure,
                run: test_make_gates_001_curated_targets_mapped,
            }],
        },
        Contract {
            id: ContractId("MAKE-GATES-002".to_string()),
            title: "make public gate visibility",
            tests: vec![TestCase {
                id: TestId("make.gates.curated_targets_public".to_string()),
                title: "curated targets are marked public in the make target registry",
                kind: TestKind::Pure,
                run: test_make_gates_002_curated_targets_public,
            }],
        },
        Contract {
            id: ContractId("MAKE-GATES-003".to_string()),
            title: "make orphan public gate removal",
            tests: vec![TestCase {
                id: TestId("make.gates.no_orphan_public_targets".to_string()),
                title: "the make target registry has no public targets outside CURATED_TARGETS",
                kind: TestKind::Pure,
                run: test_make_gates_003_no_orphan_public_targets,
            }],
        },
    ]
}

pub(super) fn contract_explain(contract_id: &str) -> Option<&'static str> {
    match contract_id {
        "MAKE-TARGETLIST-001" => {
            Some("The committed target list must declare its schema and source explicitly.")
        }
        "MAKE-NAME-001" => Some("Helper-only makefile includes must be visibly internal."),
        "MAKE-NAME-002" => Some("Files that expose curated public targets must have public names."),
        "MAKE-ART-001" => {
            Some("Curated wrappers must keep artifact output paths scoped by run id.")
        }
        "MAKE-SHELL-002" => Some("Curated wrappers must avoid multi-hop shell pipelines."),
        "MAKE-ENGINE-001" => {
            Some("Make is a wrapper layer, not the owner of direct infra tool invocations.")
        }
        "MAKE-DOCS-001" => Some("Make docs must stay small enough to remain reviewer-readable."),
        "MAKE-GATES-001" => Some("Every curated target must have a target registry mapping."),
        "MAKE-GATES-002" => {
            Some("Curated targets must be represented as public in the make target registry.")
        }
        "MAKE-GATES-003" => {
            Some("The make target registry must not expose public orphan targets outside CURATED_TARGETS.")
        }
        _ => None,
    }
}
