pub(super) fn check_configs_required_surface_paths(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let required = [
        "configs/README.md",
        "configs/INDEX.md",
        "configs/CONTRACT.md",
    ];
    let mut violations = Vec::new();
    for path in required {
        let rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "CONFIGS_REQUIRED_PATH_MISSING",
                format!("missing required configs path `{path}`"),
                "restore required configs contract files",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_configs_schema_paths_present(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let required = ["configs/schema", "configs/contracts"];
    let mut violations = Vec::new();
    for path in required {
        let rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "CONFIGS_SCHEMA_PATH_MISSING",
                format!("missing required configs schema path `{path}`"),
                "restore configs schema and contracts directories",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_make_configs_wrappers_delegate_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("make/configs.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !content.contains("BIJUX ?= bijux") || !content.contains("BIJUX_DEV_ATLAS ?=") {
        violations.push(violation(
            "MAKE_CONFIGS_BIJUX_VARIABLES_MISSING",
            "make/configs.mk must declare BIJUX and BIJUX_DEV_ATLAS variables"
                .to_string(),
            "declare BIJUX ?= bijux and BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas",
            Some(rel),
        ));
    }
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "MAKE_CONFIGS_SINGLE_LINE_RECIPE_REQUIRED",
                "make/configs.mk wrapper recipes must be single-line delegations"
                    .to_string(),
                "keep configs wrappers single-line and delegation-only",
                Some(rel),
            ));
        }
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.iter().any(|w| {
            *w == "python"
                || *w == "python3"
                || *w == "bash"
                || *w == "sh"
                || *w == "kubectl"
                || *w == "helm"
                || *w == "k6"
        }) {
            violations.push(violation(
                "MAKE_CONFIGS_DELEGATION_ONLY_VIOLATION",
                format!("make/configs.mk must remain delegation-only: `{line}`"),
                "wrapper recipes may call bijux dev atlas only",
                Some(rel),
            ));
        }
    }
    for required in ["configs-doctor:", "configs-validate:", "configs-lint:"] {
        if !content.contains(required) {
            violations.push(violation(
                "MAKE_CONFIGS_REQUIRED_TARGET_MISSING",
                format!("make/configs.mk is missing `{required}`"),
                "keep required configs delegation targets in make/configs.mk",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_ops_control_plane_doc_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel_control_plane = Path::new("ops/CONTROL_PLANE.md");
    let text_control_plane = fs::read_to_string(ctx.repo_root.join(rel_control_plane))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let rel_contract = Path::new("ops/CONTRACT.md");
    let text_contract = fs::read_to_string(ctx.repo_root.join(rel_contract))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in [
        "Control plane version:",
        "## Scope",
        "## SSOT Rules",
        "## Invariants",
        "## Effect Rules",
        "bijux dev atlas doctor",
        "check run --suite ci",
    ] {
        if !text_control_plane.contains(required) {
            violations.push(violation(
                "OPS_CONTROL_PLANE_DOC_INCOMPLETE",
                format!(
                    "ops/CONTROL_PLANE.md is missing required content `{required}`"
                ),
                "update the control plane definition document with the required invariant/entrypoint text",
                Some(rel_control_plane),
            ));
        }
    }
    for required in [
        "Ops is specification-only.",
        "Schemas under `ops/schema/` are versioned APIs",
        "Release pins are immutable after release publication",
        "_generated/` is ephemeral output only",
        "_generated.example/` is curated evidence",
        "Use `observe` as the canonical observability domain name",
        "Compatibility migrations must be timeboxed and include explicit cutoff dates",
        "Canonical directory budget:",
    ] {
        if !text_contract.contains(required) {
            violations.push(violation(
                "OPS_CONTROL_PLANE_DOC_INCOMPLETE",
                format!("ops/CONTRACT.md is missing required content `{required}`"),
                "update ops contract to keep SSOT/evolution invariants explicit and enforceable",
                Some(rel_contract),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_control_plane_naming_contract_docs(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/docs/contract.md");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in [
        "Runtime product CLI namespace: `bijux atlas <args>`",
        "Installed umbrella dispatch: `bijux dev atlas <args>`",
        "Naming contract is frozen",
    ] {
        if !text.contains(required) {
            violations.push(violation(
                "CONTROL_PLANE_NAMING_CONTRACT_MISSING",
                format!("dev control-plane contract is missing `{required}`"),
                "document the frozen runtime vs control-plane naming contract in crates/bijux-dev-atlas/docs/contract.md",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_final_dev_atlas_crate_set_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let required_dirs = ["crates/bijux-dev-atlas"];
    let mut violations = Vec::new();
    for path in required_dirs {
        let rel = Path::new(path);
        if !ctx.repo_root.join(rel).is_dir() {
            violations.push(violation(
                "DEV_ATLAS_CRATE_SET_MISSING",
                format!(
                    "required control-plane crate directory is missing: {}",
                    rel.display()
                ),
                "keep the unified dev-atlas control-plane crate present and explicitly named",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_scripting_contract_rust_control_plane_lock(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("docs/architecture/scripting.md");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in [
        "Repository automation entrypoints are Rust-native and routed through `bijux dev atlas ...`.",
        "Runtime product CLI commands are routed through `bijux atlas ...`.",
        "Python tooling documents are historical-only",
    ] {
        if !text.contains(required) {
            violations.push(violation(
                "SCRIPTING_CONTRACT_NOT_LOCKED",
                format!("scripting architecture contract is missing `{required}`"),
                "update docs/architecture/scripting.md to reflect the Rust control-plane lock and python tombstone-only policy",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_docs_ops_command_list_matches_snapshot(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/docs/OPS_COMMAND_LIST.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected = [
        "ops",
        "doctor",
        "validate",
        "render",
        "install",
        "status",
        "list-profiles",
        "explain-profile",
        "list-tools",
        "verify-tools",
        "list-actions",
        "up",
        "down",
        "clean",
        "reset",
        "pins",
        "generate",
    ]
    .join("\n");
    if current.trim() == expected.trim() {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_OPS_COMMAND_LIST_MISMATCH",
            "ops command list doc does not match canonical ops help snapshot".to_string(),
            "update crates/bijux-dev-atlas/docs/OPS_COMMAND_LIST.md to match ops --help command list",
            Some(rel),
        )])
    }
}

pub(super) fn check_docs_configs_command_list_matches_snapshot(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("crates/bijux-dev-atlas/docs/CONFIGS_COMMAND_LIST.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected = [
        "configs",
        "doctor",
        "validate",
        "lint",
        "inventory",
        "compile",
        "diff",
    ]
    .join("\n");
    if current.trim() == expected.trim() {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "DOCS_CONFIGS_COMMAND_LIST_MISMATCH",
            "configs command list doc does not match canonical configs help snapshot".to_string(),
            "update crates/bijux-dev-atlas/docs/CONFIGS_COMMAND_LIST.md to match configs --help command list",
            Some(rel),
        )])
    }
}

pub(super) fn check_ops_ssot_manifests_schema_versions(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let manifests = [
        "ops/stack/profiles.json",
        "ops/stack/generated/version-manifest.json",
        "ops/inventory/toolchain.json",
        "ops/inventory/surfaces.json",
        "ops/inventory/contracts.json",
        "ops/inventory/generated-committed-mirror.json",
    ];
    let mut violations = Vec::new();
    for path in manifests {
        let rel = Path::new(path);
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| CheckError::Failed(err.to_string()))?;
        if value.get("schema_version").is_none() {
            violations.push(violation(
                "OPS_SSOT_SCHEMA_VERSION_MISSING",
                format!("ssot manifest `{path}` must include `schema_version`"),
                "add schema_version to the SSOT manifest payload",
                Some(rel),
            ));
        }
    }
    let control_plane = Path::new("ops/CONTROL_PLANE.md");
    let control_text = fs::read_to_string(ctx.repo_root.join(control_plane))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !control_text.contains("Control plane version:") {
        violations.push(violation(
            "OPS_CONTROL_PLANE_VERSION_MISSING",
            "ops/CONTROL_PLANE.md must declare a control plane version".to_string(),
            "add `Control plane version:` line to ops/CONTROL_PLANE.md",
            Some(control_plane),
        ));
    }
    Ok(violations)
}
