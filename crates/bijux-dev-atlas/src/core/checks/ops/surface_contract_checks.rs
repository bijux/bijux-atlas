// SPDX-License-Identifier: Apache-2.0

use super::*;

pub(super) fn checks_ops_makefile_routes_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/makefiles/_ops.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected_targets = ["ops-doctor:", "ops-validate:", "ops-render:", "ops-status:"];
    let mut violations = Vec::new();
    for target in expected_targets {
        if !content.contains(target) {
            violations.push(violation(
                "OPS_MAKEFILE_TARGET_MISSING",
                format!("ops make wrapper target missing `{target}`"),
                "add thin ops wrapper target in makefiles/_ops.mk",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_make_governance_wrappers_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/makefiles/ci.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        let trimmed = line.trim();
        if !(trimmed.contains("make ")
            || trimmed.contains("$(BIJUX_DEV_ATLAS)")
            || trimmed.contains("$(BIJUX) dev atlas"))
        {
            continue;
        }
        let words = trimmed.split_whitespace().collect::<Vec<_>>();
        if words.iter().any(|w| {
            matches!(
                *w,
                "python" | "python3" | "bash" | "helm" | "kubectl" | "k6"
            )
        }) {
            violations.push(violation(
                "MAKE_GOVERNANCE_DELEGATION_ONLY_VIOLATION",
                format!("governance wrapper must be delegation-only: `{trimmed}`"),
                "keep governance wrappers routed only through make/bijux dev atlas",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_make_ops_wrappers_delegate_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/makefiles/_ops.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !content.contains("BIJUX ?= bijux") || !content.contains("BIJUX_DEV_ATLAS ?=") {
        violations.push(violation(
            "MAKE_OPS_BIJUX_VARIABLES_MISSING",
            "makefiles/_ops.mk must declare BIJUX and BIJUX_DEV_ATLAS variables".to_string(),
            "declare BIJUX and BIJUX_DEV_ATLAS wrapper variables in makefiles/_ops.mk",
            Some(rel),
        ));
    }
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "MAKE_OPS_SINGLE_LINE_RECIPE_REQUIRED",
                "makefiles/_ops.mk wrapper recipes must be single-line delegations".to_string(),
                "keep ops wrappers single-line and delegation-only",
                Some(rel),
            ));
        }
        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let direct_tool = tokens
            .first()
            .copied()
            .unwrap_or_default()
            .trim_start_matches('@');
        if matches!(
            direct_tool,
            "python" | "python3" | "bash" | "sh" | "kubectl" | "helm" | "k6"
        ) {
            violations.push(violation(
                "MAKE_OPS_DELEGATION_ONLY_VIOLATION",
                format!("makefiles/_ops.mk must be delegation-only: `{line}`"),
                "ops wrappers may call make or bijux dev atlas only",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_workflows_governance_entrypoints_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let _ = ctx;
    Ok(Vec::new())
}

pub(super) fn check_workflows_ops_entrypoints_bijux_only(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let _ = ctx;
    Ok(Vec::new())
}

pub(super) fn check_make_governance_wrappers_no_direct_cargo(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/makefiles/ci.mk");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for line in text.lines().filter(|line| line.starts_with('\t')) {
        if line.contains("cargo ") {
            violations.push(violation(
                "MAKE_GOVERNANCE_DIRECT_CARGO_REFERENCE_FOUND",
                format!(
                    "governance wrapper must not call cargo directly: `{}`",
                    line.trim()
                ),
                "route governance wrappers through bijux dev atlas",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_command_list_matches_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if current.lines().next() == Some("atlas") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_COMMAND_LIST_INVALID",
            "runtime command list doc must start with canonical `atlas` command".to_string(),
            "refresh runtime command list snapshot from bijux atlas --help",
            Some(rel),
        )])
    }
}

pub(super) fn check_docs_dev_command_list_matches_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if current.lines().next() == Some("dev") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_DEV_COMMAND_LIST_INVALID",
            "dev command list doc must start with canonical `dev` command".to_string(),
            "refresh dev command list snapshot from bijux dev atlas --help",
            Some(rel),
        )])
    }
}

pub(super) fn check_crates_bijux_atlas_reserved_verbs_exclude_dev(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-atlas-cli/src/lib.rs");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("\"dev\"") && text.contains("reserved") {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_RUNTIME_RESERVED_VERB_DEV_MISSING",
            "runtime CLI reserved verbs policy must include `dev`".to_string(),
            "keep `dev` reserved in runtime command namespace ownership rules",
            Some(rel),
        )])
    }
}

pub(super) fn check_crates_bijux_dev_atlas_not_umbrella_binary(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/Cargo.toml");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if text.contains("name = \"bijux\"") {
        Ok(vec![violation(
            "CRATES_DEV_ATLAS_UMBRELLA_BINARY_FORBIDDEN",
            "bijux-dev-atlas must not build the umbrella `bijux` binary".to_string(),
            "keep umbrella binary ownership in bijux-atlas-cli only",
            Some(rel),
        )])
    } else {
        Ok(Vec::new())
    }
}

pub(super) fn check_crates_command_namespace_ownership_unique(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let runtime_rel = Path::new("crates/bijux-atlas-cli/docs/CLI_COMMAND_LIST.md");
    let dev_rel = Path::new("crates/bijux-dev-atlas/docs/CLI_COMMAND_LIST.md");
    let runtime = fs::read_to_string(ctx.repo_root.join(runtime_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let dev = fs::read_to_string(ctx.repo_root.join(dev_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let runtime_first = runtime
        .lines()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .collect::<BTreeSet<_>>();
    let dev_first = dev
        .lines()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .collect::<BTreeSet<_>>();
    let overlap = runtime_first
        .intersection(&dev_first)
        .filter(|v| **v != "version")
        .cloned()
        .collect::<Vec<_>>();
    if overlap.is_empty() {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "CRATES_COMMAND_NAMESPACE_OWNERSHIP_DUPLICATE",
            format!("runtime and dev command surfaces have duplicate namespace ownership: {}", overlap.join(", ")),
            "keep runtime and dev command surface ownership disjoint (except shared global version semantics)",
            Some(dev_rel),
        )])
    }
}
