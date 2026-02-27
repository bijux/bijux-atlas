fn test_forbidden_extra_images(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let allowed = allowed_image_directories(&dctx.policy);
    let discovered = match image_directories_with_dockerfile(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for image in discovered {
        if !allowed.contains(&image) {
            violations.push(violation(
                "DOCKER-013",
                "docker.images.forbidden_extra",
                Some(format!("docker/images/{image}/Dockerfile")),
                Some(1),
                "docker image directory is not allowlisted",
                Some(image),
            ));
        }
    }
    if violations.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail(violations)
    }
}

fn test_effect_build_runtime_image(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let dockerfile = "docker/images/runtime/Dockerfile";
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &["build", "-f", dockerfile, "-t", &image, "."],
        "docker-build-runtime.stdout.log",
        "docker-build-runtime.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-100",
            "docker.build.runtime_image",
            Some(dockerfile.to_string()),
            Some(1),
            "docker build failed for runtime image",
            Some(truncate_for_evidence(&output.stderr)),
        )])
    }
}

fn test_effect_smoke_version(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &[
            "run",
            "--rm",
            "--entrypoint",
            "/app/bijux-atlas",
            &image,
            "version",
        ],
        "docker-smoke-version.stdout.log",
        "docker-smoke-version.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-101",
            "docker.smoke.version",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "docker smoke version command failed",
            Some(truncate_for_evidence(&output.stderr)),
        )])
    }
}

fn test_effect_smoke_help(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &[
            "run",
            "--rm",
            "--entrypoint",
            "/app/bijux-atlas",
            &image,
            "--help",
        ],
        "docker-smoke-help.stdout.log",
        "docker-smoke-help.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-101",
            "docker.smoke.help",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "docker smoke help command failed",
            Some(truncate_for_evidence(&output.stderr)),
        )])
    }
}

fn test_effect_sbom_generated(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "syft",
        &["-o", "json", &format!("docker:{image}")],
        "docker-sbom.json",
        "docker-sbom.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if !output.status.success() {
        return TestResult::Fail(vec![violation(
            "DOCKER-102",
            "docker.sbom.generated",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "syft SBOM generation failed",
            Some(truncate_for_evidence(&output.stderr)),
        )]);
    }
    match serde_json::from_slice::<Value>(&output.stdout) {
        Ok(_) => TestResult::Pass,
        Err(err) => TestResult::Fail(vec![violation(
            "DOCKER-102",
            "docker.sbom.generated",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "syft output is not valid JSON",
            Some(err.to_string()),
        )]),
    }
}

fn test_effect_scan_passes_policy(ctx: &RunContext) -> TestResult {
    let image = image_tag();
    let output = match run_command_with_artifacts(
        ctx,
        "trivy",
        &[
            "image",
            "--severity",
            "HIGH,CRITICAL",
            "--ignore-unfixed",
            "--exit-code",
            "1",
            "--format",
            "json",
            &image,
        ],
        "docker-scan.json",
        "docker-scan.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => {
            return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1));
        }
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation(
            "DOCKER-103",
            "docker.scan.severity_threshold",
            Some("docker/images/runtime/Dockerfile".to_string()),
            Some(1),
            "trivy scan failed severity threshold",
            Some(truncate_for_evidence(&output.stderr)),
        )])
    }
}

fn manifest_image_entries(repo_root: &Path) -> Result<Vec<(String, String, Vec<String>)>, String> {
    let payload = load_images_manifest(repo_root)?;
    let mut rows = Vec::new();
    for image in payload["images"].as_array().cloned().unwrap_or_default() {
        let name = image
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "docker/images.manifest.json entry missing name".to_string())?;
        let dockerfile = image
            .get("dockerfile")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "docker/images.manifest.json entry missing dockerfile".to_string())?;
        let smoke = image
            .get("smoke")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect::<Vec<_>>();
        rows.push((name.to_string(), dockerfile.to_string(), smoke));
    }
    Ok(rows)
}

fn test_effect_build_each_manifest_image(ctx: &RunContext) -> TestResult {
    let entries = match manifest_image_entries(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (name, dockerfile, _) in entries {
        let output = match run_command_with_artifacts(
            ctx,
            "docker",
            &["build", "-f", &dockerfile, "-t", &image_tag(), "."],
            &format!("docker-build-{name}.stdout.log"),
            &format!("docker-build-{name}.stderr.log"),
        ) {
            Ok(v) => v,
            Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1)),
            Err(e) => return TestResult::Error(e),
        };
        if !output.status.success() {
            violations.push(violation("DOCKER-037", "docker.effect.build_each_manifest_image", Some(dockerfile), Some(1), "docker build failed for image in manifest", Some(truncate_for_evidence(&output.stderr))));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_effect_smoke_each_manifest_image(ctx: &RunContext) -> TestResult {
    let entries = match manifest_image_entries(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (name, dockerfile, smoke) in entries {
        let tag = image_tag();
        let smoke_args = smoke.iter().map(|v| v.as_str()).collect::<Vec<_>>();
        let mut args = vec!["run", "--rm", tag.as_str()];
        args.extend(smoke_args);
        let output = match run_command_with_artifacts(
            ctx,
            "docker",
            &args,
            &format!("docker-smoke-{name}.stdout.log"),
            &format!("docker-smoke-{name}.stderr.log"),
        ) {
            Ok(v) => v,
            Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1)),
            Err(e) => return TestResult::Error(e),
        };
        if !output.status.success() {
            violations.push(violation("DOCKER-038", "docker.effect.smoke_each_manifest_image", Some(dockerfile), Some(1), "docker smoke command failed for image in manifest", Some(truncate_for_evidence(&output.stderr))));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_effect_ci_build_uses_pull_false(ctx: &RunContext) -> TestResult {
    let dockerfile = "docker/images/runtime/Dockerfile";
    let tag = image_tag();
    let args = if std::env::var_os("CI").is_some() {
        vec!["build", "--pull=false", "-f", dockerfile, "-t", tag.as_str(), "."]
    } else {
        vec!["build", "-f", dockerfile, "-t", tag.as_str(), "."]
    };
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &args,
        "docker-build-pull-policy.stdout.log",
        "docker-build-pull-policy.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1)),
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-039", "docker.effect.ci_build_uses_pull_false", Some(dockerfile.to_string()), Some(1), "docker build with CI pull policy failed", Some(truncate_for_evidence(&output.stderr)))])
    }
}

fn test_effect_build_metadata_written(ctx: &RunContext) -> TestResult {
    let output = match run_command_with_artifacts(
        ctx,
        "docker",
        &["image", "inspect", &image_tag()],
        "docker-build-metadata.json",
        "docker-build-metadata.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1)),
        Err(e) => return TestResult::Error(e),
    };
    if !output.status.success() {
        return TestResult::Fail(vec![violation("DOCKER-040", "docker.effect.build_metadata_written", Some("docker/images.manifest.json".to_string()), Some(1), "docker image inspect failed", Some(truncate_for_evidence(&output.stderr)))]);
    }
    match serde_json::from_slice::<Value>(&output.stdout) {
        Ok(_) => TestResult::Pass,
        Err(e) => TestResult::Fail(vec![violation("DOCKER-040", "docker.effect.build_metadata_written", Some("docker/images.manifest.json".to_string()), Some(1), "docker image inspect output is not valid JSON", Some(e.to_string()))]),
    }
}

fn test_effect_sbom_for_each_manifest_image(ctx: &RunContext) -> TestResult {
    let entries = match manifest_image_entries(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for (name, dockerfile, _) in entries {
        let output = match run_command_with_artifacts(
            ctx,
            "syft",
            &["-o", "json", &format!("docker:{}", image_tag())],
            &format!("docker-sbom-{name}.json"),
            &format!("docker-sbom-{name}.stderr.log"),
        ) {
            Ok(v) => v,
            Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1)),
            Err(e) => return TestResult::Error(e),
        };
        if !output.status.success() {
            violations.push(violation("DOCKER-041", "docker.effect.sbom_for_each_manifest_image", Some(dockerfile), Some(1), "syft failed for image in manifest", Some(truncate_for_evidence(&output.stderr))));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_effect_scan_output_and_threshold(ctx: &RunContext) -> TestResult {
    if std::env::var_os("CI").is_none() {
        return TestResult::Skip(
            "docker scanner contracts default to the local profile and skip outside CI"
                .to_string(),
        );
    }
    let output = match run_command_with_artifacts(
        ctx,
        "trivy",
        &["image", "--severity", "HIGH,CRITICAL", "--ignore-unfixed", "--exit-code", "1", "--format", "json", &image_tag()],
        "docker-scan-each.json",
        "docker-scan-each.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1)),
        Err(e) => return TestResult::Error(e),
    };
    if output.status.success() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-042", "docker.effect.scan_output_and_threshold", Some("docker/images.manifest.json".to_string()), Some(1), "trivy scan did not satisfy the configured severity threshold", Some(truncate_for_evidence(&output.stderr)))])
    }
}

fn test_effect_no_high_critical_without_allowlist(ctx: &RunContext) -> TestResult {
    if std::env::var_os("CI").is_none() {
        return TestResult::Skip(
            "docker scanner contracts default to the local profile and skip outside CI"
                .to_string(),
        );
    }
    let output = match run_command_with_artifacts(
        ctx,
        "trivy",
        &["image", "--severity", "HIGH,CRITICAL", "--ignore-unfixed", "--format", "json", &image_tag()],
        "docker-scan-allowlist.json",
        "docker-scan-allowlist.stderr.log",
    ) {
        Ok(v) => v,
        Err(e) if e.starts_with("SKIP_MISSING_TOOL:") => return TestResult::Skip(e.replacen("SKIP_MISSING_TOOL: ", "", 1)),
        Err(e) => return TestResult::Error(e),
    };
    if !output.status.success() {
        return TestResult::Error(truncate_for_evidence(&output.stderr));
    }
    let payload = match serde_json::from_slice::<Value>(&output.stdout) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e.to_string()),
    };
    let vuln_count = payload
        .get("Results")
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .flat_map(|row| row.get("Vulnerabilities").and_then(|v| v.as_array()).cloned().unwrap_or_default())
                .filter(|vuln| matches!(vuln.get("Severity").and_then(|v| v.as_str()), Some("HIGH" | "CRITICAL")))
                .count()
        })
        .unwrap_or(0);
    if vuln_count == 0 {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-043", "docker.effect.no_high_critical_without_allowlist", Some("docker/exceptions.json".to_string()), Some(1), "HIGH or CRITICAL vulnerabilities require an explicit allowlist justification", Some(vuln_count.to_string()))])
    }
}
