// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ContractsCommand, ContractsModeArg, DockerCommand, DockerCommonArgs, DockerPolicyCommand,
    PoliciesCommand,
};
use crate::*;
use bijux_dev_atlas::contracts;
use bijux_dev_atlas::model::CONTRACT_SCHEMA_VERSION;
use bijux_dev_atlas::policies::{canonical_policy_json, DevAtlasPolicySet};
use std::collections::VecDeque;
use std::io::{self, Write};

pub(crate) fn run_policies_command(quiet: bool, command: PoliciesCommand) -> i32 {
    let result = match command {
        PoliciesCommand::List {
            repo_root,
            format,
            out,
        } => run_policies_list(repo_root, format, out),
        PoliciesCommand::Explain {
            policy_id,
            repo_root,
            format,
            out,
        } => run_policies_explain(policy_id, repo_root, format, out),
        PoliciesCommand::Report {
            repo_root,
            format,
            out,
        } => run_policies_report(repo_root, format, out),
        PoliciesCommand::Print {
            repo_root,
            format,
            out,
        } => run_policies_print(repo_root, format, out),
        PoliciesCommand::Validate {
            repo_root,
            format,
            out,
        } => run_policies_validate(repo_root, format, out),
    };
    match result {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas policies failed: {err}");
            1
        }
    }
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let mut queue = VecDeque::from([root.to_path_buf()]);
    while let Some(dir) = queue.pop_front() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut paths = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                queue.push_back(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out
}

fn policies_inventory_rows(
    doc: &bijux_dev_atlas::policies::DevAtlasPolicySetDocument,
) -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "repo",
            "title": "repository structure and budget policy",
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "ops",
            "title": "ops registry policy",
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "compatibility",
            "title": "policy mode compatibility matrix",
            "count": doc.compatibility.len(),
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "documented_defaults",
            "title": "documented default exceptions",
            "count": doc.documented_defaults.len(),
            "schema_version": doc.schema_version.as_str()
        }),
    ]
}

pub(crate) fn run_policies_list(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let rows = policies_inventory_rows(&doc);
    let payload = serde_json::json!({
        "schema_version": 1,
        "repo_root": root.display().to_string(),
        "rows": rows,
        "text": "control-plane policies listed"
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_explain(
    policy_id: String,
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let payload = match policy_id.as_str() {
        "repo" => serde_json::json!({
            "schema_version": 1,
            "id": "repo",
            "repo_root": root.display().to_string(),
            "title": "repository structure and budget policy",
            "fields": doc.repo,
        }),
        "ops" => serde_json::json!({
            "schema_version": 1,
            "id": "ops",
            "repo_root": root.display().to_string(),
            "title": "ops registry policy",
            "fields": doc.ops,
        }),
        "compatibility" => serde_json::json!({
            "schema_version": 1,
            "id": "compatibility",
            "repo_root": root.display().to_string(),
            "title": "policy mode compatibility matrix",
            "rows": doc.compatibility,
        }),
        "documented_defaults" => serde_json::json!({
            "schema_version": 1,
            "id": "documented_defaults",
            "repo_root": root.display().to_string(),
            "title": "documented default exceptions",
            "rows": doc.documented_defaults,
        }),
        _ => {
            return Err(format!(
                "unknown policy id `{}` (expected repo|ops|compatibility|documented_defaults)",
                policy_id
            ))
        }
    };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_report(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "repo_root": root.display().to_string(),
        "policy_schema_version": doc.schema_version.as_str(),
        "mode": format!("{:?}", doc.mode).to_ascii_lowercase(),
        "policy_count": policies_inventory_rows(&doc).len(),
        "capabilities": {"fs_write": false, "subprocess": false, "network": false, "git": false},
        "report_kind": "control_plane_policies"
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_docker_command(quiet: bool, command: DockerCommand) -> i32 {
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

    let run = (|| -> Result<(String, i32), String> {
        let started = std::time::Instant::now();
        let emit = |common: &DockerCommonArgs, payload: serde_json::Value, code: i32| {
            let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
            Ok((rendered, code))
        };
        match command {
            DockerCommand::Contracts(common) => {
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "status": "ok",
                    "rows": docker_contract_rows(),
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Gates(common) => {
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "status": "ok",
                    "rows": docker_gate_rows(),
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Doctor(common) => {
                if !common.allow_subprocess {
                    return Err("docker doctor requires --allow-subprocess".to_string());
                }
                let mut rows = Vec::new();
                let (docker_code, _, docker_err) = run_subprocess(
                    &resolve_repo_root(common.repo_root.clone())?,
                    "docker",
                    &["version", "--format", "{{.Server.Version}}"],
                )?;
                rows.push(serde_json::json!({"check":"docker_version","status": if docker_code == 0 { "ok" } else { "failed" }, "stderr": docker_err}));
                let (syft_code, _, _) =
                    run_subprocess(&resolve_repo_root(common.repo_root.clone())?, "syft", &["version"])
                        .unwrap_or((1, String::new(), String::new()));
                rows.push(serde_json::json!({"check":"syft","status": if syft_code == 0 { "ok" } else { "failed" }}));
                let (trivy_code, _, _) =
                    run_subprocess(&resolve_repo_root(common.repo_root.clone())?, "trivy", &["--version"])
                        .unwrap_or((1, String::new(), String::new()));
                rows.push(serde_json::json!({"check":"trivy","status": if trivy_code == 0 { "ok" } else { "failed" }}));
                let failures = rows.iter().filter(|r| r["status"] == "failed").count();
                let payload = serde_json::json!({
                    "schema_version":1,
                    "status": if failures == 0 { "ok" } else { "failed" },
                    "rows": rows,
                    "summary": {"errors": failures, "warnings":0}
                });
                emit(&common, payload, if failures == 0 { 0 } else { 1 })
            }
            DockerCommand::Validate(common) => {
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                check_contract_gate_mapping()?;
                let rows = validate_dockerfiles(&repo_root)?;
                let violations = rows.len();
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_validate"));
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "status": if violations == 0 { "ok" } else { "failed" },
                    "text": if violations == 0 { "docker validate passed" } else { "docker validate found violations" },
                    "rows": rows,
                    "summary": {"errors": violations, "warnings": 0},
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let validate_report_path = artifact_dir.join("validate.json");
                fs::write(
                    &validate_report_path,
                    serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())? + "\n",
                )
                .map_err(|e| format!("cannot write {}: {e}", validate_report_path.display()))?;
                emit(&common, payload, if violations == 0 { 0 } else { 1 })
            }
            DockerCommand::Build(common) => {
                if !common.allow_subprocess {
                    return Err("docker build requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_build"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let (code, stdout, stderr) = run_subprocess(
                    &repo_root,
                    "docker",
                    &[
                        "build",
                        "--file",
                        "docker/images/runtime/Dockerfile",
                        "--tag",
                        &tag,
                        ".",
                    ],
                )?;
                fs::write(artifact_dir.join("build.stdout.log"), &stdout)
                    .map_err(|e| format!("cannot write build stdout log: {e}"))?;
                fs::write(artifact_dir.join("build.stderr.log"), &stderr)
                    .map_err(|e| format!("cannot write build stderr log: {e}"))?;
                let (inspect_code, inspect_stdout, _) =
                    run_subprocess(&repo_root, "docker", &["image", "inspect", &tag, "--format", "{{.Id}}"])
                        .unwrap_or((1, String::new(), String::new()));
                let (size_code, size_stdout, size_stderr) = run_subprocess(
                    &repo_root,
                    "docker",
                    &["image", "inspect", &tag, "--format", "{{.Size}}"],
                )
                .unwrap_or((1, String::new(), String::new()));
                let actual_size = size_stdout.trim().parse::<u64>().unwrap_or_default();
                let max_size = runtime_image_budget_bytes(&repo_root)?;
                let size_ok = size_code == 0 && actual_size <= max_size;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "text": "docker build completed",
                    "rows": [
                        {"action":"build","repo_root": repo_root.display().to_string(), "image_tag": tag, "image_id": inspect_stdout.trim(), "inspect_status": inspect_code},
                        {"action":"image_size","gate_id":"docker.contract.image_size","contract_id":"DOCKER-010","status": if size_ok { "ok" } else { "failed" }, "actual_bytes": actual_size, "max_bytes": max_size, "stderr": size_stderr}
                    ],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let final_code = if code == 0 && size_ok { 0 } else { 1 };
                emit(&common, payload, final_code)
            }
            DockerCommand::Check(common) => {
                let validate = run_docker_command(true, DockerCommand::Validate(common.clone()));
                emit(&common, serde_json::json!({"schema_version":1,"status": if validate == 0 { "ok" } else { "failed" }, "rows":[{"action":"check","validate_exit_code": validate}]}), validate)
            }
            DockerCommand::Smoke(common) => {
                if !common.allow_subprocess {
                    return Err("docker smoke requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_smoke"));
                let tag = image_tag_for_run(&run_id);
                let (help_code, help_out, help_err) = run_subprocess(
                    &repo_root,
                    "docker",
                    &["run", "--rm", "--entrypoint", "/app/bijux-atlas", &tag, "--help"],
                )?;
                let (version_code, version_out, version_err) = run_subprocess(
                    &repo_root,
                    "docker",
                    &["run", "--rm", "--entrypoint", "/app/bijux-atlas", &tag, "--version"],
                )?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker smoke executed",
                    "rows": [
                        {"action":"smoke_help","status": help_code, "stdout": help_out, "stderr": help_err, "contract_id":"DOCKER-007"},
                        {"action":"smoke_version","status": version_code, "stdout": version_out, "stderr": version_err, "contract_id":"DOCKER-007"}
                    ],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, if help_code == 0 && version_code == 0 { 0 } else { 1 })
            }
            DockerCommand::Scan(common) => {
                if !common.allow_subprocess {
                    return Err("docker scan requires --allow-subprocess".to_string());
                }
                if !common.allow_network {
                    return Err("docker scan requires --allow-network".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_scan"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let report_path = artifact_dir.join("scan.trivy.json");
                let report_arg = report_path.display().to_string();
                let args = [
                    "image",
                    "--format",
                    "json",
                    "--output",
                    report_arg.as_str(),
                    tag.as_str(),
                ];
                let (code, stdout, stderr) = run_subprocess(&repo_root, "trivy", &args)?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker scan executed",
                    "rows": [{"action":"scan","repo_root": repo_root.display().to_string(),"scanner":"trivy","report":report_path.display().to_string(),"stdout":stdout,"stderr":stderr,"contract_id":"DOCKER-009"}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, code)
            }
            DockerCommand::Sbom(common) => {
                if !common.allow_subprocess {
                    return Err("docker sbom requires --allow-subprocess".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_sbom"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let spdx_path = artifact_dir.join("sbom.spdx.json");
                let cyclonedx_path = artifact_dir.join("sbom.cyclonedx.json");
                let spdx_out = spdx_path.display().to_string();
                let cyclonedx_out = cyclonedx_path.display().to_string();
                let (spdx_code, spdx_stdout, spdx_stderr) = run_subprocess(
                    &repo_root,
                    "syft",
                    &[tag.as_str(), "-o", "spdx-json", "--file", spdx_out.as_str()],
                )?;
                let (cyclonedx_code, _, cyclonedx_stderr) = run_subprocess(
                    &repo_root,
                    "syft",
                    &[tag.as_str(), "-o", "cyclonedx-json", "--file", cyclonedx_out.as_str()],
                )?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker sbom executed",
                    "rows": [
                        {"action":"sbom","repo_root": repo_root.display().to_string(),"tool":"syft","format":"spdx-json","path":spdx_path.display().to_string(),"stdout":spdx_stdout,"stderr":spdx_stderr,"contract_id":"DOCKER-008"},
                        {"action":"sbom","repo_root": repo_root.display().to_string(),"tool":"syft","format":"cyclonedx-json","path":cyclonedx_path.display().to_string(),"stderr":cyclonedx_stderr,"contract_id":"DOCKER-008"}
                    ],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, if spdx_code == 0 && cyclonedx_code == 0 { 0 } else { 1 })
            }
            DockerCommand::Lock(common) => {
                if !common.allow_write {
                    return Err("docker lock requires --allow-write".to_string());
                }
                let repo_root = resolve_repo_root(common.repo_root.clone())?;
                let lock_path = repo_root.join("ops/inventory/image-digests.lock.json");
                if let Some(parent) = lock_path.parent() {
                    fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
                }
                let run_id = common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_lock"));
                let tag = image_tag_for_run(&run_id);
                let (inspect_code, inspect_stdout, inspect_stderr) =
                    run_subprocess(&repo_root, "docker", &["image", "inspect", &tag, "--format", "{{.Id}}"])
                        .unwrap_or((1, String::new(), String::new()));
                let lock_payload = serde_json::json!({
                    "schema_version": 1,
                    "kind": "docker_image_lock",
                    "images": [{"tag": tag, "digest": inspect_stdout.trim(), "inspect_status": inspect_code, "inspect_stderr": inspect_stderr}],
                    "timestamp_policy": "forbidden_by_default"
                });
                fs::write(
                    &lock_path,
                    serde_json::to_string_pretty(&lock_payload).map_err(|e| e.to_string())? + "\n",
                )
                .map_err(|e| format!("cannot write {}: {e}", lock_path.display()))?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker lockfile written under ops/inventory",
                    "rows": [{"action":"lock","path": lock_path.display().to_string()}],
                    "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&common, payload, 0)
            }
            DockerCommand::Policy { command } => match command {
                DockerPolicyCommand::Check(common) => {
                    let code = run_docker_command(true, DockerCommand::Validate(common.clone()));
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "action": "policy_check",
                        "status": if code == 0 { "ok" } else { "failed" },
                        "text": "docker policy check delegates to docker validate",
                        "rows": [{"action":"delegate","to":"docker validate","exit_code":code}],
                        "summary": {"errors": if code == 0 { 0 } else { 1 }, "warnings": 0},
                        "capabilities": {"subprocess": common.allow_subprocess, "fs_write": common.allow_write, "network": common.allow_network},
                        "duration_ms": started.elapsed().as_millis() as u64
                    });
                    emit(&common, payload, code)
                }
            },
            DockerCommand::Push(args) => {
                if !args.i_know_what_im_doing {
                    return Err("docker push requires --i-know-what-im-doing".to_string());
                }
                if !args.common.allow_subprocess {
                    return Err("docker push requires --allow-subprocess".to_string());
                }
                if !args.common.allow_network {
                    return Err("docker push requires --allow-network".to_string());
                }
                let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
                let run_id = args
                    .common
                    .run_id
                    .as_ref()
                    .map(|v| RunId::parse(v))
                    .transpose()?
                    .unwrap_or_else(|| RunId::from_seed("docker_push"));
                let tag = image_tag_for_run(&run_id);
                let artifact_dir = docker_artifact_dir(&args.common, &repo_root, &run_id);
                fs::create_dir_all(&artifact_dir)
                    .map_err(|e| format!("cannot create {}: {e}", artifact_dir.display()))?;
                let (code, stdout, stderr) = run_subprocess(&repo_root, "docker", &["push", &tag])?;
                fs::write(artifact_dir.join("push.stdout.log"), &stdout)
                    .map_err(|e| format!("cannot write push stdout log: {e}"))?;
                fs::write(artifact_dir.join("push.stderr.log"), &stderr)
                    .map_err(|e| format!("cannot write push stderr log: {e}"))?;
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "run_id": run_id.as_str(),
                    "text": "docker push executed",
                    "rows": [{"action":"push","repo_root": repo_root.display().to_string(),"image_tag":tag,"stdout":stdout,"stderr":stderr}],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write, "network": args.common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                emit(&args.common, payload, code)
            }
            DockerCommand::Release(args) => {
                if !args.i_know_what_im_doing {
                    return Err("docker release requires --i-know-what-im-doing".to_string());
                }
                if !args.common.allow_subprocess {
                    return Err("docker release requires --allow-subprocess".to_string());
                }
                if !args.common.allow_network {
                    return Err("docker release requires --allow-network".to_string());
                }
                let validate_code = run_docker_command(true, DockerCommand::Validate(args.common.clone()));
                let build_code = run_docker_command(true, DockerCommand::Build(args.common.clone()));
                let smoke_code = run_docker_command(true, DockerCommand::Smoke(args.common.clone()));
                let sbom_code = run_docker_command(true, DockerCommand::Sbom(args.common.clone()));
                let scan_code = run_docker_command(true, DockerCommand::Scan(args.common.clone()));
                let push_code = run_docker_command(
                    true,
                    DockerCommand::Push(crate::cli::DockerReleaseArgs {
                        common: args.common.clone(),
                        i_know_what_im_doing: true,
                    }),
                );
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "text": "docker release executed",
                    "rows": [
                        {"action":"validate","exit_code":validate_code},
                        {"action":"build","exit_code":build_code},
                        {"action":"smoke","exit_code":smoke_code},
                        {"action":"sbom","exit_code":sbom_code},
                        {"action":"scan","exit_code":scan_code},
                        {"action":"push","exit_code":push_code}
                    ],
                    "capabilities": {"subprocess": args.common.allow_subprocess, "fs_write": args.common.allow_write, "network": args.common.allow_network},
                    "duration_ms": started.elapsed().as_millis() as u64
                });
                let code = [validate_code, build_code, smoke_code, sbom_code, scan_code, push_code]
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or(0);
                emit(&args.common, payload, code)
            }
        }
    })();
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    let _ = writeln!(io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas docker failed: {err}");
            1
        }
    }
}

pub(crate) fn run_contracts_command(quiet: bool, command: ContractsCommand) -> i32 {
    let run = (|| -> Result<(String, i32), String> {
        match command {
            ContractsCommand::Docker(args) => {
                let repo_root = resolve_repo_root(args.repo_root)?;
                let mode = match args.mode {
                    ContractsModeArg::Static => contracts::Mode::Static,
                    ContractsModeArg::Effect => contracts::Mode::Effect,
                };
                let options = contracts::RunOptions {
                    mode,
                    allow_subprocess: args.allow_subprocess,
                    allow_network: args.allow_network,
                    fail_fast: args.fail_fast,
                    contract_filter: args.filter,
                    test_filter: args.filter_test,
                    list_only: args.list,
                    artifacts_root: args.artifacts_root,
                };
                let report = contracts::run(
                    "docker",
                    contracts::docker::contracts,
                    &repo_root,
                    &options,
                )?;
                let rendered = if args.json {
                    serde_json::to_string_pretty(&contracts::to_json(&report))
                        .map_err(|e| format!("encode contracts report failed: {e}"))?
                } else {
                    contracts::to_pretty(&report)
                };
                Ok((rendered, report.exit_code()))
            }
        }
    })();

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                if code == 0 {
                    let _ = writeln!(io::stdout(), "{rendered}");
                } else {
                    let _ = writeln!(io::stderr(), "{rendered}");
                }
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas contracts failed: {err}");
            1
        }
    }
}

pub(crate) fn run_print_boundaries_command() -> Result<(String, i32), String> {
    let payload = serde_json::json!({
        "schema_version": CONTRACT_SCHEMA_VERSION,
        "effects": [
            {"id": "fs_read", "default_allowed": true, "description": "read repository files"},
            {"id": "fs_write", "default_allowed": false, "description": "write files under artifacts only"},
            {"id": "subprocess", "default_allowed": false, "description": "execute external processes"},
            {"id": "git", "default_allowed": false, "description": "invoke git commands"},
            {"id": "network", "default_allowed": false, "description": "perform network requests"}
        ],
        "text": "effect boundaries printed"
    });
    Ok((
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?,
        0,
    ))
}

pub(crate) fn run_print_policies(repo_root: Option<PathBuf>) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let rendered = canonical_policy_json(&policies.to_document()).map_err(|err| err.to_string())?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_print(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let rendered = match format {
        FormatArg::Text => format!(
            "status: ok\nschema_version: {:?}\ncompatibility_rules: {}\ndocumented_defaults: {}",
            doc.schema_version,
            doc.compatibility.len(),
            doc.documented_defaults.len()
        ),
        FormatArg::Json => serde_json::to_string_pretty(&doc).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => serde_json::to_string(&doc).map_err(|err| err.to_string())?,
    };
    write_output_if_requested(out, &rendered)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_validate(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "repo_root": root.display().to_string(),
        "policy_schema_version": doc.schema_version,
        "compatibility_rules": doc.compatibility.len(),
        "documented_defaults": doc.documented_defaults.len(),
        "capabilities": {
            "fs_write": false,
            "subprocess": false,
            "network": false,
            "git": false
        }
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_capabilities_command(
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "text": "capabilities default-deny; commands require explicit effect flags",
        "defaults": {
            "fs_write": false,
            "subprocess": false,
            "network": false,
            "git": false
        },
        "rules": [
            {"effect": "fs_write", "policy": "explicit flag required", "flags": ["--allow-write"]},
            {"effect": "subprocess", "policy": "explicit flag required", "flags": ["--allow-subprocess"]},
            {"effect": "network", "policy": "explicit flag required", "flags": ["--allow-network"]},
            {"effect": "git", "policy": "check run only", "flags": ["--allow-git"]}
        ],
        "command_groups": [
            {"name": "check", "writes": "flag-gated", "subprocess": "flag-gated", "network": "flag-gated"},
            {"name": "docs", "writes": "flag-gated", "subprocess": "flag-gated", "network": "default-deny"},
            {"name": "configs", "writes": "flag-gated", "subprocess": "flag-gated", "network": "default-deny"},
            {"name": "ops", "writes": "flag-gated", "subprocess": "flag-gated", "network": "default-deny"}
        ]
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_version_command(
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "name": "bijux-dev-atlas",
        "version": env!("CARGO_PKG_VERSION"),
        "git_hash": option_env!("BIJUX_GIT_HASH"),
    });
    let rendered = match format {
        FormatArg::Text => {
            let version = payload
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let git_hash = payload
                .get("git_hash")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("bijux-dev-atlas {version}\ngit_hash: {git_hash}")
        }
        _ => emit_payload(format, out.clone(), &payload)?,
    };
    if matches!(format, FormatArg::Text) {
        write_output_if_requested(out, &rendered)?;
    }
    Ok((rendered, 0))
}

pub(crate) fn run_help_inventory_command(
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let payload = serde_json::json!({
        "schema_version": 1,
        "name": "bijux-dev-atlas",
        "commands": [
            {"name": "version", "kind": "leaf"},
            {"name": "help", "kind": "leaf"},
            {"name": "check", "kind": "group", "subcommands": ["registry", "list", "explain", "doctor", "run"]},
            {"name": "ops", "kind": "group"},
            {"name": "docs", "kind": "group"},
            {"name": "contracts", "kind": "group"},
            {"name": "configs", "kind": "group"},
            {"name": "build", "kind": "group"},
            {"name": "policies", "kind": "group"},
            {"name": "docker", "kind": "group"},
            {"name": "workflows", "kind": "group"},
            {"name": "gates", "kind": "group"},
            {"name": "capabilities", "kind": "leaf"}
        ]
    });
    let rendered = match format {
        FormatArg::Text => payload["commands"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|row| row.get("name").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => emit_payload(format, out.clone(), &payload)?,
    };
    if matches!(format, FormatArg::Text) {
        write_output_if_requested(out, &rendered)?;
    }
    Ok((rendered, 0))
}
