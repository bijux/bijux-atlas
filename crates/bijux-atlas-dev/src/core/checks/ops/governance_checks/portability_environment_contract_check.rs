// SPDX-License-Identifier: Apache-2.0

pub(super) fn checks_ops_portability_environment_contract(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();

    let matrix_rel = Path::new("ops/PORTABILITY_MATRIX.md");
    let matrix_path = ctx.repo_root.join(matrix_rel);
    if !matrix_path.exists() {
        violations.push(violation(
            "OPS_PORTABILITY_MATRIX_MISSING",
            "missing ops portability matrix contract `ops/PORTABILITY_MATRIX.md`".to_string(),
            "add ops/PORTABILITY_MATRIX.md with portability coverage expectations and proof mappings",
            Some(matrix_rel),
        ));
        return Ok(violations);
    }
    let matrix_text = fs::read_to_string(&matrix_path)
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", matrix_rel.display())))?;
    for required_section in [
        "## Platform Matrix",
        "## Environment Modes",
        "## Resource Pressure and Fault Simulation",
        "## Path Portability Invariants",
        "## Enforcement Links",
        "## Runtime Evidence Mapping",
    ] {
        if !matrix_text.contains(required_section) {
            violations.push(violation(
                "OPS_PORTABILITY_MATRIX_SECTION_MISSING",
                format!(
                    "portability matrix `{}` is missing required section `{required_section}`",
                    matrix_rel.display()
                ),
                "add the missing portability matrix section",
                Some(matrix_rel),
            ));
        }
    }
    for required_feature in [
        "macos-runner",
        "minimal-linux-container",
        "air-gapped-simulation",
        "degraded-stack-mode",
        "partial-dataset-mode",
        "multi-registry",
        "alternate-storage-backend",
        "cpu-limited",
        "memory-limited",
        "slow-network-simulation",
        "time-skew-simulation",
        "missing-dependency-simulation",
        "local-only",
        "remote-execution",
        "container-only-toolchain",
        "portable-path validation",
    ] {
        if !matrix_text.contains(required_feature) {
            violations.push(violation(
                "OPS_PORTABILITY_MATRIX_FEATURE_MISSING",
                format!(
                    "portability matrix `{}` must declare feature `{required_feature}`",
                    matrix_rel.display()
                ),
                "add the missing portability feature row or invariant entry",
                Some(matrix_rel),
            ));
        }
    }

    validate_k8s_install_matrix_portability(ctx, &mut violations)?;
    validate_stack_profile_portability(ctx, &mut violations)?;
    validate_load_and_observability_portability(ctx, &mut violations)?;
    validate_ops_path_portability(ctx, &mut violations)?;

    Ok(violations)
}

fn validate_k8s_install_matrix_portability(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let rel = Path::new("ops/k8s/install-matrix.json");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", rel.display())))?;
    let profiles = json
        .get("profiles")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let profile_names = profiles
        .iter()
        .filter_map(|entry| entry.get("name").and_then(|v| v.as_str()))
        .collect::<BTreeSet<_>>();
    for required in ["local", "offline", "multi-registry", "kind", "perf"] {
        if !profile_names.contains(required) {
            violations.push(violation(
                "OPS_PORTABILITY_INSTALL_PROFILE_MISSING",
                format!(
                    "k8s install matrix `{}` must include portability profile `{required}`",
                    rel.display()
                ),
                "add the required install profile to ops/k8s/install-matrix.json",
                Some(rel),
            ));
        }
    }
    Ok(())
}

fn validate_stack_profile_portability(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let rel = Path::new("ops/stack/profiles.json");
    let text = fs::read_to_string(ctx.repo_root.join(rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", rel.display())))?;
    let profiles = json
        .get("profiles")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let profile_names = profiles
        .iter()
        .filter_map(|entry| entry.get("name").and_then(|v| v.as_str()))
        .collect::<BTreeSet<_>>();
    for required in ["minimal", "kind", "perf", "ci", "dev"] {
        if !profile_names.contains(required) {
            violations.push(violation(
                "OPS_PORTABILITY_STACK_PROFILE_MISSING",
                format!(
                    "stack profiles `{}` must include portability profile `{required}`",
                    rel.display()
                ),
                "add the required stack profile to ops/stack/profiles.json",
                Some(rel),
            ));
        }
    }
    Ok(())
}

fn validate_load_and_observability_portability(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let required_paths = [
        "ops/load/scenarios/noisy-neighbor-cpu-throttle.json",
        "ops/load/scenarios/soak-30m.json",
        "ops/load/scenarios/store-outage.json",
        "ops/load/scenarios/store-outage-under-spike.json",
        "ops/load/scenarios/large-dataset-simulation.json",
        "ops/load/scenarios/redis-optional.json",
        "ops/observe/contracts/goldens/profiles.json",
        "ops/observe/contracts/goldens/offline/metrics.golden.prom",
        "ops/observe/contracts/goldens/offline/trace-structure.golden.json",
        "ops/observe/contracts/goldens/local/metrics.golden.prom",
        "ops/observe/contracts/goldens/perf/metrics.golden.prom",
    ];
    for rel_str in required_paths {
        let rel = Path::new(rel_str);
        if !ctx.repo_root.join(rel).exists() {
            violations.push(violation(
                "OPS_PORTABILITY_PROOF_ARTIFACT_MISSING",
                format!("required portability proof artifact missing `{}`", rel.display()),
                "restore the portability proof artifact or update the portability contract and checks together",
                Some(rel),
            ));
        }
    }
    Ok(())
}

fn validate_ops_path_portability(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let scan_roots = [Path::new("ops"), Path::new("docs/operations")];
    for root_rel in scan_roots {
        let root = ctx.repo_root.join(root_rel);
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            let Some(ext) = file.extension().and_then(|v| v.to_str()) else {
                continue;
            };
            if !matches!(ext, "md" | "json" | "yaml" | "yml" | "toml") {
                continue;
            }
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let text = fs::read_to_string(&file).map_err(|err| {
                CheckError::Failed(format!("read {}: {err}", rel.display()))
            })?;
            if text.contains("/Users/") || text.contains("C:\\\\") || text.contains("C:/Users/") {
                violations.push(violation(
                    "OPS_PORTABILITY_ABSOLUTE_PATH_REFERENCE_FOUND",
                    format!(
                        "portability-governed file `{}` contains a user-local absolute path reference",
                        rel.display()
                    ),
                    "use repo-relative paths in ops/docs contracts and inventories",
                    Some(rel),
                ));
            }
            if text.contains("\\ops\\") || text.contains("\\docs\\") || text.contains("\\crates\\")
            {
                violations.push(violation(
                    "OPS_PORTABILITY_WINDOWS_SEPARATOR_REFERENCE_FOUND",
                    format!(
                        "portability-governed file `{}` contains Windows-style backslash path separators",
                        rel.display()
                    ),
                    "use forward-slash repo-relative paths in authored contracts and inventories",
                    Some(rel),
                ));
            }
        }
    }
    Ok(())
}
