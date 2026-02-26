fn validate_surface_and_filesystem_policies(
    repo_root: &Path,
    inputs: &LoadedOpsInventoryValidationInputs,
    errors: &mut Vec<String>,
) {
    let inventory = &inputs.inventory;
    let mut seen_action_ids = BTreeSet::new();
    for action in &inventory.surfaces.actions {
        if action.id.trim().is_empty() {
            errors.push(format!("{OPS_SURFACES_PATH}: action id must not be empty"));
            continue;
        }
        if !seen_action_ids.insert(action.id.clone()) {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: duplicate action id `{}`",
                action.id
            ));
        }
        if action.domain.trim().is_empty() {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` has empty domain",
                action.id
            ));
        }
        if action.command.is_empty() {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` has empty command",
                action.id
            ));
        }
        let joined = action.command.join(" ");
        if joined.contains("scripts/") || joined.contains(".sh") {
            errors.push(format!(
                "{OPS_SURFACES_PATH}: action `{}` references forbidden script entrypoint `{joined}`",
                action.id
            ));
        }
    }

    for mirror in &inventory.mirror_policy.mirrors {
        if !repo_root.join(&mirror.committed).exists() {
            errors.push(format!(
                "{OPS_MIRROR_POLICY_PATH}: committed path missing `{}`",
                mirror.committed
            ));
        }
        if !mirror.source.starts_with("ops/_generated/") && !repo_root.join(&mirror.source).exists()
        {
            errors.push(format!(
                "{OPS_MIRROR_POLICY_PATH}: source path missing `{}`",
                mirror.source
            ));
        }
    }
    let sorted_mirror_keys = inventory
        .mirror_policy
        .mirrors
        .iter()
        .map(|entry| entry.committed.clone())
        .collect::<Vec<_>>();
    let mut dedup = sorted_mirror_keys.clone();
    dedup.sort();
    dedup.dedup();
    if dedup.len() != sorted_mirror_keys.len() {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: mirror committed paths must be unique"
        ));
    }
    if sorted_mirror_keys != dedup {
        errors.push(format!(
            "{OPS_MIRROR_POLICY_PATH}: mirror committed paths must be sorted for deterministic output"
        ));
    }

    let allowed_top_level: BTreeSet<&str> = [
        "_generated",
        "_generated.example",
        "_meta",
        "atlas-dev",
        "datasets",
        "docs",
        "e2e",
        "env",
        "fixtures",
        "inventory",
        "k8s",
        "load",
        "observe",
        "report",
        "schema",
        "stack",
        "tools",
        "vendor",
        "ARTIFACTS.md",
        "CONTRACT.md",
        "CONTROL_PLANE.md",
        "DIRECTORY_BUDGET_POLICY.md",
        "DOMAIN_DOCUMENT_TEMPLATE_CONTRACT.md",
        "DRIFT.md",
        "ERRORS.md",
        "GENERATED_LIFECYCLE.md",
        "GOLDEN_REFRESH_POLICY.md",
        "INDEX.md",
        "NAMING.md",
        "PORTABILITY_MATRIX.md",
        "README.md",
        "SSOT.md",
    ]
    .into_iter()
    .collect();
    if let Ok(entries) = fs::read_dir(repo_root.join("ops")) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !allowed_top_level.contains(name.as_ref()) {
                errors.push(format!("unexpected ops top-level entry `ops/{name}`"));
            }
        }
    }

    let bash_like = fs::read_dir(repo_root.join("ops"))
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .flat_map(|entry| collect_files_recursive(entry.path()))
        .filter(|path| {
            path.extension()
                .and_then(|v| v.to_str())
                .is_some_and(|ext| ext == "sh" || ext == "bash")
        })
        .collect::<Vec<_>>();
    for path in bash_like {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path.as_path())
            .display()
            .to_string();
        errors.push(format!(
            "forbidden bash helper outside rust control-plane: `{rel}`"
        ));
    }

    if repo_root.join("ops/_lib").exists() {
        errors.push("forbidden retired path exists: ops/_lib".to_string());
    }
    if repo_root.join("ops/schema/obs").exists() {
        errors.push("forbidden retired path exists: ops/schema/obs".to_string());
    }
    if let Ok(entries) = fs::read_dir(repo_root.join("ops/inventory/contracts")) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.contains("obs.") {
                errors.push(format!(
                    "forbidden retired contract fragment name in ops/inventory/contracts: `{file_name}`"
                ));
            }
        }
    }
}
