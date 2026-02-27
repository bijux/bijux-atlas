fn docker_contract_rows() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"contract_id":"DOCKER-001","name":"no-latest-tags","gate_id":"docker.contract.no_latest"}),
        serde_json::json!({"contract_id":"DOCKER-002","name":"base-images-digest-pinned","gate_id":"docker.contract.digest_pins"}),
        serde_json::json!({"contract_id":"DOCKER-003","name":"root-dockerfile-is-shim-symlink","gate_id":"docker.contract.root_symlink"}),
        serde_json::json!({"contract_id":"DOCKER-004","name":"dockerfiles-only-under-docker-images","gate_id":"docker.contract.path_scope"}),
        serde_json::json!({"contract_id":"DOCKER-005","name":"required-oci-labels-present","gate_id":"docker.contract.oci_labels"}),
        serde_json::json!({"contract_id":"DOCKER-006","name":"build-args-defaulted","gate_id":"docker.contract.build_args"}),
        serde_json::json!({"contract_id":"DOCKER-007","name":"runtime-smoke-surface","gate_id":"docker.contract.runtime_smoke"}),
        serde_json::json!({"contract_id":"DOCKER-008","name":"sbom-generated","gate_id":"docker.contract.sbom_generated"}),
        serde_json::json!({"contract_id":"DOCKER-009","name":"vuln-scan-policy","gate_id":"docker.contract.vuln_scan"}),
        serde_json::json!({"contract_id":"DOCKER-010","name":"image-size-budget","gate_id":"docker.contract.image_size"}),
    ]
}

fn docker_gate_rows() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"gate_id":"docker.contract.no_latest","command":"bijux dev atlas docker validate"}),
        serde_json::json!({"gate_id":"docker.contract.digest_pins","command":"bijux dev atlas docker validate"}),
        serde_json::json!({"gate_id":"docker.contract.root_symlink","command":"bijux dev atlas docker validate"}),
        serde_json::json!({"gate_id":"docker.contract.path_scope","command":"bijux dev atlas docker validate"}),
        serde_json::json!({"gate_id":"docker.contract.oci_labels","command":"bijux dev atlas docker validate"}),
        serde_json::json!({"gate_id":"docker.contract.build_args","command":"bijux dev atlas docker validate"}),
        serde_json::json!({"gate_id":"docker.contract.runtime_smoke","command":"bijux dev atlas docker smoke --allow-subprocess"}),
        serde_json::json!({"gate_id":"docker.contract.sbom_generated","command":"bijux dev atlas docker sbom --allow-subprocess"}),
        serde_json::json!({"gate_id":"docker.contract.vuln_scan","command":"bijux dev atlas docker scan --allow-subprocess --allow-network"}),
        serde_json::json!({"gate_id":"docker.contract.image_size","command":"bijux dev atlas docker build --allow-subprocess"}),
    ]
}

fn check_contract_gate_mapping() -> Result<(), String> {
    let contract_gate_ids = docker_contract_rows()
        .into_iter()
        .filter_map(|row| row["gate_id"].as_str().map(ToString::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    let gate_ids = docker_gate_rows()
        .into_iter()
        .filter_map(|row| row["gate_id"].as_str().map(ToString::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    if contract_gate_ids != gate_ids {
        return Err(format!(
            "docker contract to gate mapping mismatch: contracts={contract_gate_ids:?} gates={gate_ids:?}"
        ));
    }
    Ok(())
}

fn image_tag_for_run(run_id: &RunId) -> String {
    format!("bijux-atlas:{}", run_id.as_str())
}

fn docker_artifact_dir(common: &DockerCommonArgs, repo_root: &Path, run_id: &RunId) -> PathBuf {
    let root = common
        .artifacts_root
        .as_ref()
        .map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                repo_root.join(p)
            }
        })
        .unwrap_or_else(|| repo_root.join("artifacts"));
    root.join(run_id.as_str()).join("docker")
}

fn run_subprocess(
    repo_root: &Path,
    program: &str,
    args: &[&str],
) -> Result<(i32, String, String), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to run `{program}`: {e}"))?;
    Ok((
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

fn extract_copy_sources(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with("COPY ") || trimmed.contains("--from=") {
        return None;
    }
    let rest = trimmed.trim_start_matches("COPY ").trim();
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }
    Some(
        tokens[..tokens.len() - 1]
            .iter()
            .map(|s| s.trim_matches('"').to_string())
            .collect(),
    )
}

fn all_dockerfiles(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let images_root = repo_root.join("docker/images");
    if images_root.exists() {
        for path in walk_files(&images_root) {
            if path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s == "Dockerfile")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn validate_dockerfiles(repo_root: &Path) -> Result<Vec<serde_json::Value>, String> {
    let policy_path = repo_root.join("docker/policy.json");
    let policy_text = fs::read_to_string(&policy_path)
        .map_err(|e| format!("failed to read {}: {e}", policy_path.display()))?;
    let policy: serde_json::Value = serde_json::from_str(&policy_text)
        .map_err(|e| format!("failed to parse {}: {e}", policy_path.display()))?;
    let exceptions = policy["allow_tagged_images_exceptions"]
        .as_array()
        .ok_or_else(|| "digest pinning policy missing allowlist array".to_string())?
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();
    let allowed_network_tokens = policy["build_network_policy"]["allowed_tokens"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let forbidden_network_tokens = policy["build_network_policy"]["forbidden_tokens"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let required_labels = policy["required_oci_labels"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    let docker_root = repo_root.join("docker");
    for file in walk_files(&docker_root) {
        let rel = file
            .strip_prefix(repo_root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let allowed = rel == "docker/README.md"
            || rel == "docker/CONTRACT.md"
            || rel == "docker/policy.json"
            || rel.starts_with("docker/images/")
            || rel.starts_with("docker/fixtures/");
        if !allowed {
            rows.push(serde_json::json!({
                "contract_id":"DOCKER-004",
                "gate_id":"docker.contract.path_scope",
                "kind":"docker_allowed_file_violation",
                "file": rel,
                "line": 1
            }));
        }
        if rel.ends_with(".md") && rel != "docker/README.md" && rel != "docker/CONTRACT.md" {
            rows.push(serde_json::json!({
                "contract_id":"DOCKER-004",
                "gate_id":"docker.contract.path_scope",
                "kind":"docker_markdown_forbidden",
                "file": rel,
                "line": 1
            }));
        }
        if rel.ends_with("/README.md") && rel != "docker/README.md" {
            rows.push(serde_json::json!({
                "contract_id":"DOCKER-004",
                "gate_id":"docker.contract.path_scope",
                "kind":"nested_readme_forbidden",
                "file": rel,
                "line": 1
            }));
        }
        if rel.ends_with("/CONTRACT.md") && rel != "docker/CONTRACT.md" {
            rows.push(serde_json::json!({
                "contract_id":"DOCKER-004",
                "gate_id":"docker.contract.path_scope",
                "kind":"nested_contract_forbidden",
                "file": rel,
                "line": 1
            }));
        }
    }

    let docs_root = repo_root.join("docs");
    if docs_root.exists() {
        for file in walk_files(&docs_root) {
            let rel = file
                .strip_prefix(repo_root)
                .unwrap_or(&file)
                .display()
                .to_string();
            if !rel.ends_with(".md") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&file) else {
                continue;
            };
            for (idx, line) in text.lines().enumerate() {
                if line.contains("docker/contracts/") {
                    rows.push(serde_json::json!({
                        "contract_id":"DOCKER-004",
                        "gate_id":"docker.contract.path_scope",
                        "kind":"docs_docker_link_sanity_violation",
                        "file": rel,
                        "line": idx + 1,
                        "evidence": "docker/contracts/"
                    }));
                }
            }
        }
    }

    let root_dockerfile = repo_root.join("Dockerfile");
    if !root_dockerfile.exists() {
        rows.push(serde_json::json!({
            "contract_id":"DOCKER-003",
            "gate_id":"docker.contract.root_symlink",
            "kind":"root_dockerfile_missing",
            "file":"Dockerfile",
            "line": 1
        }));
    } else {
        let meta = fs::symlink_metadata(&root_dockerfile)
            .map_err(|e| format!("failed to stat {}: {e}", root_dockerfile.display()))?;
        if !meta.file_type().is_symlink() {
            rows.push(serde_json::json!({
                "contract_id":"DOCKER-003",
                "gate_id":"docker.contract.root_symlink",
                "kind":"root_dockerfile_not_symlink",
                "file":"Dockerfile",
                "line": 1
            }));
        }
    }

    let dockerfiles = all_dockerfiles(repo_root)?;
    for dockerfile in dockerfiles {
        let rel = dockerfile
            .strip_prefix(repo_root)
            .unwrap_or(&dockerfile)
            .display()
            .to_string();
        if !rel.starts_with("docker/images/") {
            rows.push(serde_json::json!({
                "contract_id":"DOCKER-004",
                "gate_id":"docker.contract.path_scope",
                "kind":"dockerfile_outside_scope",
                "file": rel,
                "line": 1
            }));
        }
        let text = fs::read_to_string(&dockerfile)
            .map_err(|e| format!("failed to read {}: {e}", dockerfile.display()))?;
        let mut labels_present = std::collections::BTreeSet::new();
        for (idx, line) in text.lines().enumerate() {
            if let Some(srcs) = extract_copy_sources(line) {
                for src in srcs {
                    if src == "." || src.starts_with('/') {
                        continue;
                    }
                    if !repo_root.join(&src).exists() {
                        rows.push(serde_json::json!({
                            "contract_id":"DOCKER-004",
                            "gate_id":"docker.contract.path_scope",
                            "kind":"copy_source_missing",
                            "file": rel,
                            "line": idx + 1,
                            "evidence": src
                        }));
                    }
                }
            }

            let trimmed = line.trim();
            if trimmed.starts_with("LABEL ") {
                for key in &required_labels {
                    if trimmed.contains(key) {
                        labels_present.insert(key.clone());
                    }
                }
            }
            if trimmed.starts_with("ARG ")
                && !trimmed.contains('=')
                && trimmed
                    .split_whitespace()
                    .nth(1)
                    .is_some_and(|name| name == "RUST_VERSION" || name == "IMAGE_VERSION")
            {
                rows.push(serde_json::json!({
                    "contract_id":"DOCKER-006",
                    "gate_id":"docker.contract.build_args",
                    "kind":"required_arg_missing_default",
                    "file": rel,
                    "line": idx + 1,
                    "evidence": trimmed
                }));
            }
            if trimmed.starts_with("RUN ")
                && !allowed_network_tokens
                    .iter()
                    .any(|token| trimmed.contains(token))
                && forbidden_network_tokens
                    .iter()
                    .any(|token| trimmed.contains(token))
            {
                rows.push(serde_json::json!({
                    "contract_id":"DOCKER-004",
                    "gate_id":"docker.contract.path_scope",
                    "kind":"build_network_policy_violation",
                    "file": rel,
                    "line": idx + 1,
                    "evidence": trimmed
                }));
            }
            if trimmed.starts_with("ADD ") && (trimmed.contains("http://") || trimmed.contains("https://")) {
                rows.push(serde_json::json!({
                    "contract_id":"DOCKER-004",
                    "gate_id":"docker.contract.path_scope",
                    "kind":"add_remote_url_forbidden",
                    "file": rel,
                    "line": idx + 1,
                    "evidence": trimmed
                }));
            }
            if trimmed.starts_with("RUN ")
                && (trimmed.contains("curl") || trimmed.contains("wget"))
                && trimmed.contains('|')
                && trimmed.contains("sh")
            {
                rows.push(serde_json::json!({
                    "contract_id":"DOCKER-004",
                    "gate_id":"docker.contract.path_scope",
                    "kind":"curl_pipe_sh_forbidden",
                    "file": rel,
                    "line": idx + 1,
                    "evidence": trimmed
                }));
            }
            if !trimmed.starts_with("FROM ") {
                continue;
            }
            let from_spec = trimmed.split_whitespace().nth(1).ok_or_else(|| {
                format!("invalid FROM line in {}: {}", dockerfile.display(), trimmed)
            })?;
            let uses_latest = from_spec.ends_with(":latest") || from_spec == "latest";
            let is_digest_pinned = from_spec.contains("@sha256:");
            let is_allowlisted = exceptions.iter().any(|e| e == &from_spec);
            if uses_latest {
                rows.push(serde_json::json!({
                    "contract_id":"DOCKER-001",
                    "gate_id":"docker.contract.no_latest",
                    "kind":"latest_tag_forbidden",
                    "file": rel,
                    "line": idx + 1,
                    "evidence": from_spec
                }));
            }
            if !is_digest_pinned && !is_allowlisted {
                rows.push(serde_json::json!({
                    "contract_id":"DOCKER-002",
                    "gate_id":"docker.contract.digest_pins",
                    "kind":"digest_pin_required",
                    "file": rel,
                    "line": idx + 1,
                    "evidence": from_spec
                }));
            }
        }
        for label in &required_labels {
            if !labels_present.contains(label) {
                rows.push(serde_json::json!({
                    "contract_id":"DOCKER-005",
                    "gate_id":"docker.contract.oci_labels",
                    "kind":"required_label_missing",
                    "file": rel,
                    "line": 1,
                    "evidence": label
                }));
            }
        }
    }
    Ok(rows)
}

fn runtime_image_budget_bytes(repo_root: &Path) -> Result<u64, String> {
    let policy_path = repo_root.join("docker/policy.json");
    let policy_text = fs::read_to_string(&policy_path)
        .map_err(|e| format!("failed to read {}: {e}", policy_path.display()))?;
    let policy: serde_json::Value = serde_json::from_str(&policy_text)
        .map_err(|e| format!("failed to parse {}: {e}", policy_path.display()))?;
    policy["runtime_image_max_bytes"]
        .as_u64()
        .ok_or_else(|| "docker policy missing runtime_image_max_bytes".to_string())
}
