fn test_no_pip_install_without_hashes(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if args.contains("pip install") && !args.contains("--require-hashes") {
                    violations.push(violation("DOCKER-044", "docker.run.no_pip_install_without_hashes", Some(rel.clone()), Some(ins.line), "pip install requires --require-hashes or a locked strategy", Some(ins.args)));
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_no_cargo_install_without_version(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if args.contains("cargo install") && !args.contains("--version") {
                    violations.push(violation("DOCKER-045", "docker.run.no_cargo_install_without_version", Some(rel.clone()), Some(ins.line), "cargo install must pin a version", Some(ins.args)));
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_no_go_install_latest(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword == "RUN" {
                let args = ins.args.to_ascii_lowercase();
                if args.contains("go install") && args.contains("@latest") {
                    violations.push(violation("DOCKER-046", "docker.run.no_go_install_latest", Some(rel.clone()), Some(ins.line), "go install must not use @latest", Some(ins.args)));
                }
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_markdown_surface_only_root_docs(ctx: &RunContext) -> TestResult {
    test_dir_allowed_markdown(ctx)
}

fn test_contract_registry_export_matches(ctx: &RunContext) -> TestResult {
    let expected = match render_contract_registry_json(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let path = ctx.repo_root.join("docker/docker.contracts.json");
    let actual = match std::fs::read_to_string(&path) {
        Ok(v) => v.trim().to_string(),
        Err(e) => return TestResult::Error(format!("read {} failed: {e}", path.display())),
    };
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-049", "docker.registry.export_matches_generated", Some("docker/docker.contracts.json".to_string()), Some(1), "docker contract registry export drifted from generated output", None)])
    }
}

fn test_contract_gate_map_matches(ctx: &RunContext) -> TestResult {
    let expected = match render_contract_gate_map_json(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let path = ctx.repo_root.join("ops/inventory/docker-contract-gate-map.json");
    let actual = match std::fs::read_to_string(&path) {
        Ok(v) => v.trim().to_string(),
        Err(e) => return TestResult::Error(format!("read {} failed: {e}", path.display())),
    };
    if actual == expected {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-050", "docker.gate_map.matches_generated", Some("ops/inventory/docker-contract-gate-map.json".to_string()), Some(1), "docker contract gate map drifted from generated output", None)])
    }
}

fn test_exceptions_registry_schema(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("docker/exceptions.json");
    let payload = match load_json(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Fail(vec![violation("DOCKER-051", "docker.exceptions.schema_valid", Some("docker/exceptions.json".to_string()), Some(1), &e, None)]),
    };
    let mut violations = Vec::new();
    if payload.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
        violations.push(violation("DOCKER-051", "docker.exceptions.schema_valid", Some("docker/exceptions.json".to_string()), Some(1), "exceptions registry must set schema_version=1", None));
    }
    if payload.get("exceptions").and_then(|v| v.as_array()).is_none() {
        violations.push(violation("DOCKER-051", "docker.exceptions.schema_valid", Some("docker/exceptions.json".to_string()), Some(1), "exceptions registry must define an exceptions array", None));
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_exceptions_minimal(ctx: &RunContext) -> TestResult {
    let path = ctx.repo_root.join("docker/exceptions.json");
    let payload = match load_json(&path) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let known = match contracts(&ctx.repo_root) {
        Ok(v) => v.into_iter().map(|contract| contract.id.0).collect::<BTreeSet<_>>(),
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for entry in payload["exceptions"].as_array().cloned().unwrap_or_default() {
        let contract_id = entry.get("contract_id").and_then(|v| v.as_str()).unwrap_or("");
        let expires_on = entry.get("expires_on").and_then(|v| v.as_str()).unwrap_or("");
        let justification = entry.get("justification").and_then(|v| v.as_str()).unwrap_or("");
        if !known.contains(contract_id) {
            violations.push(violation("DOCKER-052", "docker.exceptions.minimal_entries", Some("docker/exceptions.json".to_string()), Some(1), "exception must cite a valid contract id", Some(contract_id.to_string())));
        }
        let valid_date = expires_on.len() == 10
            && expires_on.chars().enumerate().all(|(idx, ch)| match idx {
                4 | 7 => ch == '-',
                _ => ch.is_ascii_digit(),
            });
        if !valid_date {
            violations.push(violation("DOCKER-052", "docker.exceptions.minimal_entries", Some("docker/exceptions.json".to_string()), Some(1), "exception must set expires_on in YYYY-MM-DD format", Some(expires_on.to_string())));
        }
        if justification.trim().is_empty() {
            violations.push(violation("DOCKER-052", "docker.exceptions.minimal_entries", Some("docker/exceptions.json".to_string()), Some(1), "exception must include a justification", Some(contract_id.to_string())));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_scan_profile_policy(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    let local = dctx
        .policy
        .get("profiles")
        .and_then(|v| v.get("local"))
        .and_then(|v| v.get("allow_scan_skip"))
        .and_then(|v| v.as_bool());
    let ci = dctx
        .policy
        .get("profiles")
        .and_then(|v| v.get("ci"))
        .and_then(|v| v.get("allow_scan_skip"))
        .and_then(|v| v.as_bool());
    if local != Some(true) {
        violations.push(violation("DOCKER-053", "docker.scan.profile_policy", Some("docker/policy.json".to_string()), Some(1), "docker policy must allow scan skips for the local profile", None));
    }
    if ci != Some(false) {
        violations.push(violation("DOCKER-053", "docker.scan.profile_policy", Some("docker/policy.json".to_string()), Some(1), "docker policy must forbid scan skips for the ci profile", None));
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_runtime_engine_policy(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let engine = dctx
        .policy
        .get("runtime_engine")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if engine == "docker" {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-054", "docker.runtime.engine_policy", Some("docker/policy.json".to_string()), Some(1), "docker policy must explicitly set runtime_engine to `docker` until podman support is implemented", Some(engine.to_string()))])
    }
}

fn test_airgap_build_policy_stub(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let airgap = dctx.policy.get("airgap_build");
    let declared = airgap.and_then(|v| v.get("declared")).and_then(|v| v.as_bool());
    let policy = airgap.and_then(|v| v.get("policy")).and_then(|v| v.as_str());
    if declared == Some(false) && policy == Some("stub") {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-055", "docker.build.airgap_policy_stub", Some("docker/policy.json".to_string()), Some(1), "docker policy must carry an explicit airgap build stub with declared=false and policy=stub", None)])
    }
}

fn test_multi_registry_push_policy_stub(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let policy = dctx.policy.get("multi_registry_push");
    let declared = policy.and_then(|v| v.get("declared")).and_then(|v| v.as_bool());
    let registries = policy.and_then(|v| v.get("registries")).and_then(|v| v.as_array());
    if declared == Some(false) && registries.is_some() {
        TestResult::Pass
    } else {
        TestResult::Fail(vec![violation("DOCKER-056", "docker.push.multi_registry_policy_stub", Some("docker/policy.json".to_string()), Some(1), "docker policy must carry an explicit multi-registry push stub", None)])
    }
}

fn test_downloaded_assets_are_verified(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let require_digest_pins = match load_ctx(&ctx.repo_root) {
        Ok(v) => v.policy.get("downloaded_assets").and_then(|v| v.get("require_digest_pins")).and_then(|v| v.as_bool()).unwrap_or(false),
        Err(e) => return TestResult::Error(e),
    };
    if !require_digest_pins {
        return TestResult::Fail(vec![violation("DOCKER-057", "docker.run.downloaded_assets_are_verified", Some("docker/policy.json".to_string()), Some(1), "docker policy must require digest pinning for downloaded assets", None)]);
    }
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "RUN" {
                continue;
            }
            let args = ins.args.to_ascii_lowercase();
            let downloads_asset = args.contains("curl ") || args.contains("wget ");
            let verifies_checksum = args.contains("sha256sum")
                || args.contains("sha512sum")
                || args.contains("openssl dgst")
                || args.contains("checksum");
            if downloads_asset && !verifies_checksum {
                violations.push(violation("DOCKER-057", "docker.run.downloaded_assets_are_verified", Some(rel.clone()), Some(ins.line), "downloaded assets must be verified with a checksum in the same RUN instruction", Some(ins.args)));
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_vendored_binaries_allowlisted(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let allowed = dctx
        .policy
        .get("vendored_binaries")
        .and_then(|v| v.get("allow"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for file in walk_files(&ctx.repo_root.join("docker")).unwrap_or_default() {
        let rel = file
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&file)
            .display()
            .to_string()
            .replace('\\', "/");
        let is_candidate = [".bin", ".tgz", ".tar", ".tar.gz", ".zip"]
            .iter()
            .any(|suffix| rel.ends_with(suffix));
        if is_candidate && !allowed.contains(&rel) {
            violations.push(violation("DOCKER-058", "docker.vendored_binaries.allowlisted", Some(rel.clone()), Some(1), "vendored binary artifacts must be explicitly allowlisted in docker policy", None));
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_no_curl_pipe_shell(ctx: &RunContext) -> TestResult {
    let rows = match dockerfiles_with_instructions(ctx) { Ok(v) => v, Err(e) => return TestResult::Error(e) };
    let mut violations = Vec::new();
    for (rel, instructions) in rows {
        for ins in instructions {
            if ins.keyword != "RUN" {
                continue;
            }
            let args = ins.args.to_ascii_lowercase();
            let network_pipe = (args.contains("curl ") || args.contains("wget ")) && args.contains('|');
            let shell_sink = args.contains(" bash") || args.ends_with("bash") || args.contains(" sh") || args.ends_with("sh");
            if network_pipe && shell_sink {
                violations.push(violation("DOCKER-059", "docker.run.no_curl_pipe_shell", Some(rel.clone()), Some(ins.line), "curl or wget output must not be piped into a shell interpreter", Some(ins.args)));
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

fn test_dockerfiles_canonical_whitespace(ctx: &RunContext) -> TestResult {
    let dctx = match load_ctx(&ctx.repo_root) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(e),
    };
    let mut violations = Vec::new();
    for dockerfile in dctx.dockerfiles {
        let rel = dockerfile
            .strip_prefix(&ctx.repo_root)
            .unwrap_or(&dockerfile)
            .display()
            .to_string()
            .replace('\\', "/");
        let content = match std::fs::read_to_string(&dockerfile) {
            Ok(v) => v,
            Err(e) => return TestResult::Error(format!("read {} failed: {e}", dockerfile.display())),
        };
        for (idx, line) in content.lines().enumerate() {
            if line.contains('\t') {
                violations.push(violation("DOCKER-060", "docker.dockerfiles.canonical_whitespace", Some(rel.clone()), Some(idx + 1), "dockerfiles must not contain tab characters", None));
            }
            if line.ends_with(' ') {
                violations.push(violation("DOCKER-060", "docker.dockerfiles.canonical_whitespace", Some(rel.clone()), Some(idx + 1), "dockerfiles must not contain trailing whitespace", None));
            }
        }
    }
    if violations.is_empty() { TestResult::Pass } else { TestResult::Fail(violations) }
}

