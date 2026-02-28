
pub(super) fn checks_ops_no_scripts_areas_or_xtask_refs(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let targets = [
        Path::new("make/makefiles/ops.mk"),
        Path::new(".github/workflows/ci-pr.yml"),
        Path::new("ops/README.md"),
        Path::new("ops/INDEX.md"),
    ];
    let needles = ["scripts/areas", "xtask"];
    let mut violations = Vec::new();
    for rel in targets {
        let path = ctx.repo_root.join(rel);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('!') && trimmed.contains("rg -n") {
                continue;
            }
            for needle in needles {
                if trimmed.contains(needle) {
                    violations.push(violation(
                        "OPS_LEGACY_REFERENCE_FOUND",
                        format!(
                            "forbidden retired reference `{needle}` found in {}: `{trimmed}`",
                            rel.display()
                        ),
                        "remove scripts/areas and xtask references from ops-owned surfaces",
                        Some(rel),
                    ));
                }
            }
        }
    }
    let canonical_docs = [
        Path::new("ops/CONTRACT.md"),
        Path::new("ops/INDEX.md"),
        Path::new("ops/README.md"),
        Path::new("docs/operations/ops-system/INDEX.md"),
    ];
    for rel in canonical_docs {
        let path = ctx.repo_root.join(rel);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        if content.contains("ops/obs/") {
            violations.push(violation(
                "OPS_LEGACY_REFERENCE_FOUND",
                format!(
                    "legacy observability path `ops/obs/` found in canonical document {}",
                    rel.display()
                ),
                "use canonical `ops/observe/` path and keep migration notes in dedicated migration docs only",
                Some(rel),
            ));
        }
    }
    let ops_docs_root = ctx.repo_root.join("ops");
    if ops_docs_root.exists() {
        for file in walk_files(&ops_docs_root) {
            if file.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            if content.contains("ops/schema/obs/") || content.contains("ops/obs/") {
                violations.push(violation(
                    "OPS_LEGACY_REFERENCE_FOUND",
                    format!(
                        "retired observability path reference found in {}",
                        rel.display()
                    ),
                    "replace legacy observability paths with canonical ops/observe paths",
                    Some(rel),
                ));
            }
        }
    }
    Ok(violations)
}

pub(super) fn checks_ops_artifacts_gitignore_policy(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let rel = Path::new(".gitignore");
    let path = ctx.repo_root.join(rel);
    let content = fs::read_to_string(&path).map_err(|err| CheckError::Failed(err.to_string()))?;
    if content
        .lines()
        .any(|line| line.trim() == "artifacts/" || line.trim() == "/artifacts/")
    {
        Ok(Vec::new())
    } else {
        Ok(vec![violation(
            "OPS_ARTIFACTS_GITIGNORE_MISSING",
            "artifacts/ must be ignored in .gitignore".to_string(),
            "add `artifacts/` to .gitignore",
            Some(rel),
        )])
    }
}

pub(super) fn checks_ops_workflow_routes_dev_atlas(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
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
        if !text.contains("RUN_ID:") {
            if text.contains("ISO_ROOT: artifacts/isolates/") {
                for required_tmp in ["TMPDIR:", "TMP:", "TEMP:"] {
                    if !text.contains(required_tmp) {
                        violations.push(violation(
                            "WORKFLOW_ISOLATION_TEMP_ENV_MISSING",
                            format!(
                                "workflow `{}` defines ISO_ROOT isolation but is missing `{required_tmp}` temp environment binding",
                                rel.display()
                            ),
                            "bind TMPDIR, TMP, and TEMP under the workflow isolate tmp directory",
                            Some(rel),
                        ));
                    }
                }
            }
            continue;
        }
        if !text.contains("github.run_attempt") {
            violations.push(violation(
                "WORKFLOW_RUN_ID_ATTEMPT_SUFFIX_MISSING",
                format!(
                    "workflow `{}` RUN_ID must include github.run_attempt for retry-safe isolation",
                    rel.display()
                ),
                "append `${{ github.run_attempt }}` to workflow RUN_ID definitions",
                Some(rel),
            ));
        }
        if !text.contains("ISO_ROOT: artifacts/isolates/") {
            violations.push(violation(
                "WORKFLOW_ARTIFACT_ISOLATION_ROOT_MISSING",
                format!(
                    "workflow `{}` defines RUN_ID but is missing ISO_ROOT under artifacts/isolates/",
                    rel.display()
                ),
                "declare ISO_ROOT under artifacts/isolates/<lane> for workflows that emit run-scoped artifacts",
                Some(rel),
            ));
        }
        for required_tmp in ["TMPDIR:", "TMP:", "TEMP:"] {
            if !text.contains(required_tmp) {
                violations.push(violation(
                    "WORKFLOW_ISOLATION_TEMP_ENV_MISSING",
                    format!(
                        "workflow `{}` is missing `{required_tmp}` temp environment binding",
                        rel.display()
                    ),
                    "bind TMPDIR, TMP, and TEMP under the workflow isolate tmp directory",
                    Some(rel),
                ));
            }
        }
        if text.contains("TMPDIR:") && !text.contains("TMPDIR: artifacts/isolates/") {
            violations.push(violation(
                "WORKFLOW_ISOLATION_TMPDIR_LAYOUT_MISSING",
                format!(
                    "workflow `{}` must bind TMPDIR under its isolate tmp path",
                    rel.display()
                ),
                "set TMPDIR/TMP/TEMP to artifacts/isolates/<lane>/tmp",
                Some(rel),
            ));
        }
        if !text.contains("artifacts/${RUN_ID}/") {
            violations.push(violation(
                "WORKFLOW_RUN_ID_ARTIFACT_LAYOUT_MISSING",
                format!(
                    "workflow `{}` defines RUN_ID but does not write summary/log/report paths under artifacts/${{RUN_ID}}/",
                    rel.display()
                ),
                "write workflow reports and logs under artifacts/${RUN_ID}/...",
                Some(rel),
            ));
        }
        if !text.contains("rm -rf \"artifacts/${RUN_ID}\"")
            || !text.contains("mkdir -p \"artifacts/${RUN_ID}\"")
        {
            violations.push(violation(
                "WORKFLOW_RUN_ID_ARTIFACT_CLEANUP_MISSING",
                format!(
                    "workflow `{}` must clean and recreate artifacts/${{RUN_ID}} before lane execution",
                    rel.display()
                ),
                "add a shell step that runs `rm -rf \"artifacts/${RUN_ID}\"` and `mkdir -p \"artifacts/${RUN_ID}\"`",
                Some(rel),
            ));
        }
        if text.contains("actions/upload-artifact@")
            && !text.contains("path: artifacts/${{ env.RUN_ID }}")
        {
            violations.push(violation(
                "WORKFLOW_ARTIFACT_UPLOAD_PATH_NOT_RUN_SCOPED",
                format!(
                    "workflow `{}` uploads artifacts but not from path artifacts/${{ env.RUN_ID }}",
                    rel.display()
                ),
                "upload the run-scoped artifact directory path artifacts/${{ env.RUN_ID }}",
                Some(rel),
            ));
        }
    }
    Ok(violations)
}

