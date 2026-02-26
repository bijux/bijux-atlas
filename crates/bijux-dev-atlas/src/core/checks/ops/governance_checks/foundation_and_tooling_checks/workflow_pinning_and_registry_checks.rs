pub(super) fn checks_ops_workflows_github_actions_pinned(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let toolchain_rel = Path::new("ops/inventory/toolchain.json");
    let toolchain_path = ctx.repo_root.join(toolchain_rel);
    let toolchain_text = fs::read_to_string(&toolchain_path)
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", toolchain_rel.display())))?;
    let toolchain_json: serde_json::Value = serde_json::from_str(&toolchain_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", toolchain_rel.display())))?;
    let Some(actions_obj) = toolchain_json
        .get("github_actions")
        .and_then(|v| v.as_object())
    else {
        return Ok(vec![violation(
            "OPS_TOOLCHAIN_ACTIONS_PIN_SET_MISSING",
            "ops/inventory/toolchain.json is missing github_actions pin set".to_string(),
            "declare workflow action refs and immutable SHAs under github_actions",
            Some(toolchain_rel),
        )]);
    };

    let mut allowed_shas: BTreeMap<String, String> = BTreeMap::new();
    for (name, entry) in actions_obj {
        let Some(sha) = entry.get("sha").and_then(|v| v.as_str()) else {
            continue;
        };
        allowed_shas.insert(name.clone(), sha.to_string());
    }

    let workflows_root = ctx.repo_root.join(".github/workflows");
    if !workflows_root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for file in walk_files(&workflows_root) {
        if file.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(&file);
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("- uses:") && !trimmed.starts_with("uses:") {
                continue;
            }
            let Some((_, spec_raw)) = trimmed.split_once(':') else {
                continue;
            };
            let spec = spec_raw.trim();
            if spec.starts_with("docker://") {
                continue;
            }
            let Some((action_path, sha)) = spec.rsplit_once('@') else {
                violations.push(violation(
                    "WORKFLOW_ACTION_PIN_MISSING",
                    format!(
                        "workflow `{}` line {} uses action `{spec}` without an immutable SHA",
                        rel.display(),
                        line_idx + 1
                    ),
                    "pin GitHub Actions to full 40-hex commit SHAs",
                    Some(rel),
                ));
                continue;
            };
            if sha.len() != 40 || !sha.chars().all(|ch| ch.is_ascii_hexdigit()) {
                violations.push(violation(
                    "WORKFLOW_ACTION_NOT_SHA_PINNED",
                    format!(
                        "workflow `{}` line {} action `{action_path}` is pinned to `{sha}`, not a 40-hex SHA",
                        rel.display(),
                        line_idx + 1
                    ),
                    "replace action version tags with immutable commit SHAs",
                    Some(rel),
                ));
                continue;
            }
            match allowed_shas.get(action_path) {
                Some(expected_sha) if expected_sha == sha => {}
                Some(expected_sha) => violations.push(violation(
                    "WORKFLOW_ACTION_SHA_MISMATCH",
                    format!(
                        "workflow `{}` line {} action `{action_path}` sha `{sha}` does not match ops/inventory/toolchain.json expected `{expected_sha}`",
                        rel.display(),
                        line_idx + 1
                    ),
                    "sync workflow action pin to the canonical toolchain github_actions pin set",
                    Some(rel),
                )),
                None => violations.push(violation(
                    "WORKFLOW_ACTION_NOT_ALLOWLISTED",
                    format!(
                        "workflow `{}` line {} action `{action_path}` is not declared in ops/inventory/toolchain.json github_actions",
                        rel.display(),
                        line_idx + 1
                    ),
                    "declare every workflow action in ops/inventory/toolchain.json github_actions before use",
                    Some(rel),
                )),
            }
        }
    }
    Ok(violations)
}

pub(super) fn checks_ops_image_references_digest_pinned(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();

    let toolchain_rel = Path::new("ops/inventory/toolchain.json");
    let toolchain_path = ctx.repo_root.join(toolchain_rel);
    if let Ok(text) = fs::read_to_string(&toolchain_path) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(images) = value.get("images").and_then(|v| v.as_object()) {
                for (name, image_ref) in images {
                    if name == "generated_by" {
                        continue;
                    }
                    let Some(image_ref) = image_ref.as_str() else {
                        continue;
                    };
                    if !image_ref.contains("@sha256:") {
                        violations.push(violation(
                            "OPS_IMAGE_TAG_ONLY_REFERENCE",
                            format!(
                                "ops/inventory/toolchain.json images.{name} is not digest pinned: `{image_ref}`"
                            ),
                            "pin all canonical ops image references to immutable digests",
                            Some(toolchain_rel),
                        ));
                    }
                }
            }
        }
    }

    let compose_rel = Path::new("ops/observe/pack/compose/docker-compose.yml");
    let compose_path = ctx.repo_root.join(compose_rel);
    if let Ok(text) = fs::read_to_string(&compose_path) {
        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("image: ") {
                continue;
            }
            let image_ref = trimmed.trim_start_matches("image: ").trim();
            if !image_ref.contains("@sha256:") {
                violations.push(violation(
                    "OPS_COMPOSE_IMAGE_TAG_ONLY_REFERENCE",
                    format!(
                        "{} line {} image is not digest pinned: `{image_ref}`",
                        compose_rel.display(),
                        line_idx + 1
                    ),
                    "pin compose images to immutable digests",
                    Some(compose_rel),
                ));
            }
        }
    }

    Ok(violations)
}

pub(super) fn check_ops_internal_registry_consistency(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let path = ctx.repo_root.join(crate::core::DEFAULT_REGISTRY_PATH);
    let output = ctx
        .adapters
        .process
        .run(
            "git",
            &[
                "status".to_string(),
                "--short".to_string(),
                path.display().to_string(),
            ],
            ctx.repo_root,
        )
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if output == 0 {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "OPS_INTERNAL_REGISTRY_GIT_STATUS_FAILED",
            "unable to query git status for registry".to_string(),
            "ensure git is available and repository is valid",
            Some(Path::new(crate::core::DEFAULT_REGISTRY_PATH)),
        )])
    }
}

