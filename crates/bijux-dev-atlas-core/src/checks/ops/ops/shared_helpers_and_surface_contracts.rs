fn violation(code: &str, message: String, hint: &str, path: Option<&Path>) -> Violation {
    Violation {
        schema_version: bijux_dev_atlas_model::schema_version(),
        code: ViolationId::parse(&code.to_ascii_lowercase()).expect("valid violation id"),
        message,
        hint: Some(hint.to_string()),
        path: path.map(|p| ArtifactPath::parse(&p.display().to_string()).expect("valid path")),
        line: None,
        severity: Severity::Error,
    }
}

fn read_dir_entries(path: &Path) -> Vec<PathBuf> {
    match fs::read_dir(path) {
        Ok(entries) => entries.filter_map(Result::ok).map(|e| e.path()).collect(),
        Err(_) => Vec::new(),
    }
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in read_dir_entries(&dir) {
            if entry.is_dir() {
                stack.push(entry);
            } else if entry.is_file() {
                out.push(entry);
            }
        }
    }
    out.sort();
    out
}

fn check_ops_surface_manifest(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let manifest = Path::new("configs/ops/ops-surface-manifest.json");
    let surface = Path::new("ops/inventory/surfaces.json");
    let mut violations = Vec::new();
    if !ctx.adapters.fs.exists(ctx.repo_root, manifest) {
        violations.push(violation(
            "OPS_SURFACE_MANIFEST_MISSING",
            "missing configs/ops/ops-surface-manifest.json".to_string(),
            "restore ops surface manifest",
            Some(manifest),
        ));
    }
    if !ctx.adapters.fs.exists(ctx.repo_root, surface) {
        violations.push(violation(
            "OPS_SURFACE_INVENTORY_MISSING",
            "missing ops/inventory/surfaces.json".to_string(),
            "regenerate inventory surfaces",
            Some(surface),
        ));
    }
    Ok(violations)
}

fn checks_ops_tree_contract(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let required = [
        "ops/CONTRACT.md",
        "ops/INDEX.md",
        "ops/ERRORS.md",
        "ops/README.md",
    ];
    let mut violations = Vec::new();
    for path in required {
        let rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_TREE_REQUIRED_PATH_MISSING",
                format!("missing required ops path `{path}`"),
                "restore the required ops contract file",
                Some(rel),
            ));
        }
    }
    let canonical_dirs = [
        "inventory",
        "schema",
        "env",
        "stack",
        "k8s",
        "observe",
        "load",
        "datasets",
        "e2e",
        "report",
        "_generated",
        "_generated.example",
    ];
    for dir in canonical_dirs {
        let rel = Path::new("ops").join(dir);
        if !ctx.adapters.fs.exists(ctx.repo_root, &rel) {
            violations.push(violation(
                "OPS_CANONICAL_DIRECTORY_MISSING",
                format!("missing canonical ops directory `{}`", rel.display()),
                "restore the canonical ops directory set",
                Some(&rel),
            ));
            continue;
        }
        for required_file in ["README.md", "OWNER.md", "REQUIRED_FILES.md"] {
            let target = rel.join(required_file);
            if !ctx.adapters.fs.exists(ctx.repo_root, &target) {
                violations.push(violation(
                    "OPS_CANONICAL_DIRECTORY_REQUIRED_FILE_MISSING",
                    format!(
                        "missing required file `{}` in canonical ops directory",
                        target.display()
                    ),
                    "add required README/OWNER/REQUIRED_FILES marker files for canonical ops directories",
                    Some(&target),
                ));
            }
        }
        let full = ctx.repo_root.join(&rel);
        let has_any_entry = fs::read_dir(&full)
            .ok()
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false);
        if !has_any_entry {
            violations.push(violation(
                "OPS_CANONICAL_DIRECTORY_EMPTY",
                format!("canonical ops directory is empty: `{}`", rel.display()),
                "add required marker files and committed contract content",
                Some(&rel),
            ));
        }
    }
    let allowed_top_level_dirs = BTreeSet::from([
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
    ]);
    for entry in read_dir_entries(&ctx.repo_root.join("ops")) {
        if !entry.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !allowed_top_level_dirs.contains(name) {
            let rel = Path::new("ops").join(name);
            violations.push(violation(
                "OPS_TOP_LEVEL_DIRECTORY_FORBIDDEN",
                format!("non-canonical top-level ops directory found: `{}`", rel.display()),
                "remove stray directories or update contract and checks if the directory is intentional",
                Some(&rel),
            ));
        }
    }

    let env_required = [
        "ops/env/base/overlay.json",
        "ops/env/dev/overlay.json",
        "ops/env/ci/overlay.json",
        "ops/env/prod/overlay.json",
    ];
    for path in env_required {
        let rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_ENV_OVERLAY_FILE_MISSING",
                format!("missing required environment overlay file `{path}`"),
                "add the required overlay.json file for each canonical environment",
                Some(rel),
            ));
        }
    }

    for file in walk_files(&ctx.repo_root.join("ops/env")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        let rel_str = rel.display().to_string();
        if rel_str.ends_with(".sh")
            || rel_str.ends_with(".bash")
            || rel_str.ends_with(".py")
            || rel_str.ends_with(".rs")
        {
            violations.push(violation(
                "OPS_ENV_RUNTIME_LOGIC_FORBIDDEN",
                format!(
                    "runtime logic file is forbidden in ops/env: `{}`",
                    rel.display()
                ),
                "keep ops/env overlays as pure data only",
                Some(rel),
            ));
            continue;
        }
        if rel_str.ends_with(".json") {
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                violations.push(violation(
                    "OPS_ENV_OVERLAY_INVALID_JSON",
                    format!("overlay file is not valid JSON: `{}`", rel.display()),
                    "fix JSON syntax in environment overlay file",
                    Some(rel),
                ));
                continue;
            };
            if value.get("schema_version").is_none() {
                violations.push(violation(
                    "OPS_ENV_OVERLAY_SCHEMA_VERSION_MISSING",
                    format!("overlay file missing schema_version: `{}`", rel.display()),
                    "add schema_version field to overlay.json",
                    Some(rel),
                ));
            }
            if value.get("values").and_then(|v| v.as_object()).is_none() {
                violations.push(violation(
                    "OPS_ENV_OVERLAY_VALUES_MISSING",
                    format!("overlay file missing object `values`: `{}`", rel.display()),
                    "add values object to overlay.json",
                    Some(rel),
                ));
            }
        }
    }

    if let Ok(merged) = merged_env_overlay(ctx.repo_root) {
        for required in [
            "namespace",
            "cluster_profile",
            "allow_write",
            "allow_subprocess",
            "network_mode",
        ] {
            if !merged.contains_key(required) {
                violations.push(violation(
                    "OPS_ENV_OVERLAY_MERGE_INCOMPLETE",
                    format!("merged env overlay is missing required key `{required}`"),
                    "ensure base and environment overlays provide required keys after merge",
                    Some(Path::new("ops/env")),
                ));
            }
        }
    }
    Ok(violations)
}

fn merged_env_overlay(
    repo_root: &Path,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let base = parse_overlay_values(repo_root, "ops/env/base/overlay.json")?;
    let mut merged = base;
    for rel in [
        "ops/env/dev/overlay.json",
        "ops/env/ci/overlay.json",
        "ops/env/prod/overlay.json",
    ] {
        let current = parse_overlay_values(repo_root, rel)?;
        for (key, value) in current {
            merged.insert(key, value);
        }
    }
    Ok(merged)
}

fn parse_overlay_values(
    repo_root: &Path,
    rel: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let path = repo_root.join(rel);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let value = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    value
        .get("values")
        .and_then(|v| v.as_object())
        .cloned()
        .ok_or_else(|| format!("{rel}: missing `values` object"))
}

fn extract_required_files_yaml_block(content: &str) -> Option<String> {
    let mut in_yaml = false;
    let mut yaml_block = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "```yaml" {
            in_yaml = true;
            continue;
        }
        if trimmed == "```" && in_yaml {
            break;
        }
        if in_yaml {
            yaml_block.push_str(line);
            yaml_block.push('\n');
        }
    }
    if yaml_block.trim().is_empty() {
        None
    } else {
        Some(yaml_block)
    }
}

fn collect_json_string_values(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => out.push(s.to_string()),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_json_string_values(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for item in map.values() {
                collect_json_string_values(item, out);
            }
        }
        _ => {}
    }
}

fn checks_ops_generated_readonly_markers(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let policy_rel = Path::new("ops/inventory/generated-committed-mirror.json");
    let policy_path = ctx.repo_root.join(policy_rel);
    let policy_text =
        fs::read_to_string(&policy_path).map_err(|err| CheckError::Failed(err.to_string()))?;
    let policy_json: serde_json::Value =
        serde_json::from_str(&policy_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let mut allowlisted = BTreeSet::new();
    if let Some(entries) = policy_json
        .get("allow_runtime_compat")
        .and_then(|v| v.as_array())
    {
        for entry in entries {
            if let Some(path) = entry.as_str() {
                allowlisted.insert(path.to_string());
            }
        }
    }
    if let Some(entries) = policy_json.get("mirrors").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(path) = entry.get("committed").and_then(|v| v.as_str()) {
                allowlisted.insert(path.to_string());
            }
        }
    }

    let roots = ["ops/_generated.example"];
    let mut violations = Vec::new();
    for root in roots {
        let dir = ctx.repo_root.join(root);
        if !dir.exists() {
            continue;
        }
        for file in walk_files(&dir) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
            let rel_str = rel.display().to_string();
            if !allowlisted.contains(&rel_str) {
                violations.push(violation(
                    "OPS_GENERATED_FILE_ALLOWLIST_MISSING",
                    format!("generated mirror file `{}` is not declared in mirror policy", rel_str),
                    "declare generated mirror files in ops/inventory/generated-committed-mirror.json",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

