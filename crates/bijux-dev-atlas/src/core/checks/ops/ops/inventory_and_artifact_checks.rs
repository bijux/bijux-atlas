fn checks_ops_manifest_integrity(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let manifests: [(&str, &[&str]); 4] = [
        (
            "ops/inventory/surfaces.json",
            &["schema_version", "entrypoints"],
        ),
        ("ops/inventory/contracts.json", &["schema_version"]),
        ("ops/inventory/drills.json", &["schema_version"]),
        ("ops/inventory/gates.json", &["schema_version", "gates"]),
    ];
    let mut violations = Vec::new();
    for (path, required_keys) in manifests {
        let rel = Path::new(path);
        let target = ctx.repo_root.join(rel);
        let Ok(text) = fs::read_to_string(&target) else {
            violations.push(violation(
                "OPS_MANIFEST_MISSING",
                format!("missing required manifest `{path}`"),
                "restore required inventory manifest",
                Some(rel),
            ));
            continue;
        };
        let parsed = serde_json::from_str::<serde_json::Value>(&text);
        let Ok(value) = parsed else {
            violations.push(violation(
                "OPS_MANIFEST_INVALID_JSON",
                format!("manifest `{path}` is not valid JSON"),
                "fix JSON syntax in inventory manifest",
                Some(rel),
            ));
            continue;
        };
        for key in required_keys {
            if value.get(*key).is_none() {
                violations.push(violation(
                    "OPS_MANIFEST_REQUIRED_KEY_MISSING",
                    format!("manifest `{path}` is missing key `{key}`"),
                    "add the required key to the manifest payload",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

fn checks_ops_surface_inventory(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let index_rel = Path::new("ops/INDEX.md");
    let index = ctx.repo_root.join(index_rel);
    let index_text =
        fs::read_to_string(&index).map_err(|err| CheckError::Failed(err.to_string()))?;
    let required_entries = [
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
    let listed_dirs: BTreeSet<String> = index_text
        .lines()
        .filter(|line| line.trim_start().starts_with("- `ops/"))
        .filter_map(|line| line.split("`ops/").nth(1))
        .filter_map(|tail| tail.split('/').next())
        .map(|name| name.to_string())
        .collect();

    let mut violations = Vec::new();
    for dir in required_entries {
        if !listed_dirs.contains(dir) {
            violations.push(violation(
                "OPS_INDEX_DIRECTORY_MISSING",
                format!("ops/INDEX.md does not list ops directory `{dir}`"),
                "regenerate ops index so directories are listed",
                Some(index_rel),
            ));
        }
    }
    let listed_order = index_text
        .lines()
        .filter(|line| line.trim_start().starts_with("- `ops/"))
        .filter_map(|line| line.split("`ops/").nth(1))
        .filter_map(|tail| tail.split('/').next())
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let expected_order = required_entries
        .iter()
        .map(|v| (*v).to_string())
        .collect::<Vec<_>>();
    if listed_order != expected_order {
        violations.push(violation(
            "OPS_INDEX_DIRECTORY_ORDER_INVALID",
            format!(
                "ops/INDEX.md canonical directory order mismatch: listed={listed_order:?} expected={expected_order:?}"
            ),
            "list canonical ops directories in stable contract order",
            Some(index_rel),
        ));
    }
    Ok(violations)
}

fn checks_ops_artifacts_not_tracked(ctx: &CheckContext<'_>) -> Result<Vec<Violation>, CheckError> {
    let evidence_root = ctx.repo_root.join("ops/_evidence");
    if !evidence_root.exists() {
        return Ok(Vec::new());
    }
    let files = walk_files(&evidence_root);
    let tracked_like = files
        .into_iter()
        .filter(|path| path.file_name().and_then(|v| v.to_str()) != Some(".gitkeep"))
        .collect::<Vec<_>>();
    if tracked_like.is_empty() {
        Ok(Vec::new())
    } else {
        let first = tracked_like[0]
            .strip_prefix(ctx.repo_root)
            .unwrap_or(&tracked_like[0]);
        Ok(vec![violation(
            "OPS_ARTIFACTS_POLICY_VIOLATION",
            format!(
                "ops evidence directory contains committed file `{}`",
                first.display()
            ),
            "remove files under ops/_evidence and keep runtime output under artifacts/",
            Some(Path::new("ops/_evidence")),
        )])
    }
}

fn checks_ops_retired_artifact_path_references_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let allowlist = [
        "ops/ARTIFACTS.md",
        "ops/CONTRACT.md",
        "ops/_generated.example/control-plane.snapshot.md",
    ];
    let scan_roots = [
        Path::new("ops"),
        Path::new("crates/bijux-dev-atlas/src"),
        Path::new("crates/bijux-dev-atlas/docs"),
    ];
    for scan_root in scan_roots {
        let full_root = ctx.repo_root.join(scan_root);
        if !full_root.exists() {
            continue;
        }
        for path in walk_files(&full_root) {
            let Ok(rel) = path.strip_prefix(ctx.repo_root) else {
                continue;
            };
            let rel_str = rel.display().to_string();
            if allowlist.iter().any(|allowed| *allowed == rel_str) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            if text.contains("ops/_artifacts") {
                violations.push(violation(
                    "OPS_RETIRED_ARTIFACT_PATH_REFERENCE",
                    format!("retired artifact path reference found in `{}`", rel.display()),
                    "replace ops/_artifacts paths with canonical artifacts/ layout",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

fn checks_ops_runtime_output_roots_under_ops_absent(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    for retired_root in ["ops/_artifacts", "ops/_evidence"] {
        let rel = Path::new(retired_root);
        let full = ctx.repo_root.join(rel);
        if !full.exists() {
            continue;
        }
        let files = walk_files(&full)
            .into_iter()
            .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some(".gitkeep"))
            .collect::<Vec<_>>();
        if files.is_empty() {
            continue;
        }
        violations.push(violation(
            "OPS_RUNTIME_OUTPUT_ROOT_UNDER_OPS_FORBIDDEN",
            format!(
                "retired runtime output root under ops contains files: `{}`",
                retired_root
            ),
            "store runtime outputs under artifacts/ and keep ops/ for authored or curated files only",
            Some(rel),
        ));
    }
    let runtime_source_roots = [
        Path::new("crates/bijux-dev-atlas/src/commands/ops/execution_runtime_mod"),
        Path::new("crates/bijux-dev-atlas/src/commands/ops/runtime_mod"),
    ];
    for root_rel in runtime_source_roots {
        let root = ctx.repo_root.join(root_rel);
        if !root.exists() {
            continue;
        }
        for file in walk_files(&root) {
            if file.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            if rel.display().to_string().contains("/tests.rs") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            for line in text.lines() {
                let trimmed = line.trim();
                let writes_ops_tree = (trimmed.contains("create_dir_all(")
                    || trimmed.contains("std::fs::write(")
                    || trimmed.contains("fs::write("))
                    && trimmed.contains("join(\"ops/");
                if writes_ops_tree {
                    violations.push(violation(
                        "OPS_RUNTIME_SOURCE_WRITES_UNDER_OPS_FORBIDDEN",
                        format!(
                            "runtime source `{}` writes under ops tree in line `{trimmed}`",
                            rel.display()
                        ),
                        "write runtime outputs under artifacts/ and restrict ops/ writes to explicit governance/manifests outside runtime execution modules",
                        Some(rel),
                    ));
                    break;
                }
            }
        }
    }
    Ok(violations)
}
