pub(super) fn check_configs_required_surface_paths(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let required = [
        "configs/README.md",
        "configs/CONTRACT.md",
        "configs/NAMING.md",
        "configs/OWNERS.md",
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
    let required = ["configs/schemas/registry", "configs/schemas/contracts"];
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
    let rel = Path::new("makes/configs.mk");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    if !content.contains("BIJUX ?= bijux") || !content.contains("BIJUX_DEV_ATLAS ?=") {
        violations.push(violation(
            "MAKE_CONFIGS_BIJUX_VARIABLES_MISSING",
            "makes/configs.mk must declare BIJUX and BIJUX_DEV_ATLAS variables"
                .to_string(),
            "declare BIJUX ?= bijux and BIJUX_DEV_ATLAS ?= $(BIJUX) dev atlas",
            Some(rel),
        ));
    }
    for line in content.lines().filter(|line| line.starts_with('\t')) {
        if line.trim_end().ends_with('\\') {
            violations.push(violation(
                "MAKE_CONFIGS_SINGLE_LINE_RECIPE_REQUIRED",
                "makes/configs.mk wrapper recipes must be single-line delegations"
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
                format!("makes/configs.mk must remain delegation-only: `{line}`"),
                "wrapper recipes may call bijux dev atlas only",
                Some(rel),
            ));
        }
    }
    for required in ["configs-doctor:", "configs-validate:", "configs-lint:"] {
        if !content.contains(required) {
            violations.push(violation(
                "MAKE_CONFIGS_REQUIRED_TARGET_MISSING",
                format!("makes/configs.mk is missing `{required}`"),
                "keep required configs delegation targets in makes/configs.mk",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_ops_control_plane_doc_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel_readme = Path::new("ops/README.md");
    let text_readme = fs::read_to_string(ctx.repo_root.join(rel_readme))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let rel_ssot = Path::new("ops/SSOT.md");
    let text_ssot = fs::read_to_string(ctx.repo_root.join(rel_ssot))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let rel_contract = Path::new("ops/CONTRACT.md");
    let text_contract = fs::read_to_string(ctx.repo_root.join(rel_contract))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut violations = Vec::new();
    for required in [
        "Machine validation entrypoint:",
        "Human walkthroughs and procedures live in `docs/",
        "Root Docs",
    ] {
        if !text_readme.contains(required) {
            violations.push(violation(
                "OPS_ROOT_README_INCOMPLETE",
                format!("ops/README.md is missing required content `{required}`"),
                "keep the root ops README aligned with the live control-plane entrypoints",
                Some(rel_readme),
            ));
        }
    }
    for required in [
        "## Scope",
        "## Durable Rules",
        "## Machine Authorities",
        "## Evidence",
        "## Minimal Release Surface",
    ] {
        if !text_contract.contains(required) {
            violations.push(violation(
                "OPS_CONTRACT_DOC_INCOMPLETE",
                format!("ops/CONTRACT.md is missing required content `{required}`"),
                "update ops/CONTRACT.md to reflect the live ops authorities and invariants",
                Some(rel_contract),
            ));
        }
    }
    for required in ["## Allowed Root Markdown", "## Forbidden Markdown Shape"] {
        if !text_ssot.contains(required) {
            violations.push(violation(
                "OPS_SSOT_DOC_INCOMPLETE",
                format!("ops/SSOT.md is missing required content `{required}`"),
                "keep ops/SSOT.md aligned with the five-root-doc markdown policy",
                Some(rel_ssot),
            ));
        }
    }
    Ok(violations)
}

pub(super) fn check_control_plane_naming_contract_docs(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("docs/bijux-dev-atlas-docs/contract.md");
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
                "document the frozen runtime vs control-plane naming contract in bijux-dev-atlas-docs/contract.md",
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
    let rel = Path::new("docs/_internal/generated/ops-command-list.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected = [
        "ops",
        "logs",
        "describe",
        "events",
        "resources",
        "kind",
        "helm",
        "list",
        "explain",
        "stack",
        "k8s",
        "profiles",
        "profile",
        "load",
        "datasets",
        "e2e",
        "scenario",
        "obs",
        "schema",
        "inventory-domain",
        "report-domain",
        "evidence",
        "diagnose",
        "drills",
        "tools",
        "suite",
        "doctor",
        "validate",
        "graph",
        "inventory",
        "docs",
        "docs-verify",
        "conformance",
        "report",
        "helm-env",
        "readiness",
        "render",
        "install",
        "smoke",
        "status",
        "list-profiles",
        "explain-profile",
        "list-tools",
        "verify-tools",
        "list-actions",
        "plan",
        "package",
        "release-plan",
        "install-plan",
        "up",
        "down",
        "clean",
        "cleanup",
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
            "update docs/_internal/generated/ops-command-list.md to match ops --help command list",
            Some(rel),
        )])
    }
}

pub(super) fn check_docs_configs_command_list_matches_snapshot(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new("docs/_internal/generated/configs-command-list.md");
    let current = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected = [
        "configs",
        "print",
        "list",
        "graph",
        "explain",
        "verify",
        "doctor",
        "validate",
        "lint",
        "fmt",
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
            "update docs/_internal/generated/configs-command-list.md to match configs --help command list",
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
    let ssot_rel = Path::new("ops/SSOT.md");
    let ssot_text = fs::read_to_string(ctx.repo_root.join(ssot_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if !ssot_text.contains("ops/SSOT.md") {
        violations.push(violation(
            "OPS_SSOT_ROOT_DOC_REFERENCE_MISSING",
            "ops/SSOT.md should explicitly list itself in the allowed root markdown set"
                .to_string(),
            "keep ops/SSOT.md explicit about the full five-doc root markdown surface",
            Some(ssot_rel),
        ));
    }
    Ok(violations)
}
