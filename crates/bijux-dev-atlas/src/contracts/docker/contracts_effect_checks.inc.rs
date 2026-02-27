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
            "--version",
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

