// SPDX-License-Identifier: Apache-2.0

use crate::cli::{SecurityCommand, SecurityScanArtifactsArgs, SecurityValidateArgs};
use crate::{emit_payload, resolve_repo_root};
use std::fs;
use std::path::{Path, PathBuf};

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn report_path(root: &Path) -> Result<PathBuf, String> {
    let path = root.join("artifacts/security/security-threat-model.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

fn named_report_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    let path = root.join("artifacts/security").join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(path)
}

fn ensure_json(path: &Path) -> Result<(), String> {
    let _: serde_json::Value = read_json(path)?;
    Ok(())
}

fn collect_json_key_strings(value: &serde_json::Value, key: &str, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (name, inner) in map {
                if name == key {
                    if let Some(text) = inner.as_str() {
                        out.push(text.to_string());
                    }
                }
                collect_json_key_strings(inner, key, out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_json_key_strings(item, key, out);
            }
        }
        _ => {}
    }
}

fn parse_requirement_indexes(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            for prefix in ["--index-url", "--extra-index-url"] {
                if trimmed.starts_with(prefix) {
                    let remainder = trimmed.trim_start_matches(prefix).trim();
                    if !remainder.is_empty() {
                        return Some(
                            remainder
                                .split_whitespace()
                                .next()
                                .unwrap_or_default()
                                .to_string(),
                        );
                    }
                }
            }
            None
        })
        .collect()
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| matches!(idx, 4 | 7) || byte.is_ascii_digit())
}

pub(crate) fn run_security_command(
    _quiet: bool,
    command: SecurityCommand,
) -> Result<(String, i32), String> {
    match command {
        SecurityCommand::Validate(args) => run_security_validate(args),
        SecurityCommand::Compliance { command } => match command {
            crate::cli::SecurityComplianceCommand::Validate(args) => {
                run_security_compliance_validate(args)
            }
        },
        SecurityCommand::ScanArtifacts(args) => run_security_scan_artifacts(args),
    }
}

fn list_files_recursive(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        if !path.exists() {
            continue;
        }
        for entry in fs::read_dir(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else {
                files.push(entry_path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn scan_matches(
    root: &Path,
    dir: &Path,
    patterns: &[String],
) -> Result<Vec<serde_json::Value>, String> {
    let mut matches = Vec::new();
    for path in list_files_recursive(dir)? {
        let content = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        for pattern in patterns {
            if content.contains(pattern) {
                matches.push(serde_json::json!({
                    "path": path.strip_prefix(root).unwrap_or(&path).display().to_string(),
                    "pattern": pattern
                }));
            }
        }
    }
    Ok(matches)
}

fn run_security_validate(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let assets_path = root.join("security/threat-model/assets.yaml");
    let threats_path = root.join("security/threat-model/threats.yaml");
    let mitigations_path = root.join("security/threat-model/mitigations.yaml");
    let controls_path = root.join("security/compliance/controls.yaml");
    let auth_model_path = root.join("configs/security/auth-model.yaml");
    let principals_path = root.join("configs/security/principals.yaml");
    let actions_path = root.join("configs/security/actions.yaml");
    let resources_path = root.join("configs/security/resources.yaml");
    let policy_path = root.join("configs/security/policy.yaml");
    let asset_schema_path = root.join("configs/contracts/security/assets.schema.json");
    let threats_schema_path = root.join("configs/contracts/security/threats.schema.json");
    let mitigations_schema_path = root.join("configs/contracts/security/mitigations.schema.json");
    let controls_schema_path = root.join("configs/contracts/security/controls.schema.json");
    let auth_model_schema_path = root.join("configs/contracts/security/auth-model.schema.json");
    let principals_schema_path = root.join("configs/contracts/security/principals.schema.json");
    let actions_schema_path = root.join("configs/contracts/security/actions.schema.json");
    let resources_schema_path = root.join("configs/contracts/security/resources.schema.json");
    let policy_schema_path = root.join("configs/contracts/security/policy.schema.json");
    let secrets_schema_path = root.join("configs/contracts/security/secrets.schema.json");
    let redaction_schema_path = root.join("configs/contracts/security/redaction.schema.json");
    let forbidden_patterns_schema_path =
        root.join("configs/contracts/security/forbidden-patterns.schema.json");
    let dependency_policy_schema_path =
        root.join("configs/contracts/security/dependency-source-policy.schema.json");
    let github_actions_exceptions_schema_path =
        root.join("configs/contracts/security/github-actions-exceptions.schema.json");
    let signing_policy_schema_path =
        root.join("configs/contracts/release/signing-policy.schema.json");
    let secrets_path = root.join("configs/security/secrets.json");
    let redaction_path = root.join("configs/security/redaction.json");
    let forbidden_patterns_path = root.join("configs/security/forbidden-patterns.json");
    let dependency_policy_path = root.join("configs/security/dependency-source-policy.json");
    let signing_policy_path = root.join("release/signing/policy.yaml");

    ensure_json(&asset_schema_path)?;
    ensure_json(&threats_schema_path)?;
    ensure_json(&mitigations_schema_path)?;
    ensure_json(&controls_schema_path)?;
    ensure_json(&auth_model_schema_path)?;
    ensure_json(&principals_schema_path)?;
    ensure_json(&actions_schema_path)?;
    ensure_json(&resources_schema_path)?;
    ensure_json(&policy_schema_path)?;
    ensure_json(&secrets_schema_path)?;
    ensure_json(&redaction_schema_path)?;
    ensure_json(&forbidden_patterns_schema_path)?;
    ensure_json(&dependency_policy_schema_path)?;
    ensure_json(&github_actions_exceptions_schema_path)?;
    ensure_json(&signing_policy_schema_path)?;

    let assets = read_yaml(&assets_path)?;
    let threats = read_yaml(&threats_path)?;
    let mitigations = read_yaml(&mitigations_path)?;
    let controls = read_yaml(&controls_path)?;
    let auth_model = read_yaml(&auth_model_path)?;
    let principals = read_yaml(&principals_path)?;
    let actions = read_yaml(&actions_path)?;
    let resources = read_yaml(&resources_path)?;
    let policy = read_yaml(&policy_path)?;
    let secrets = read_json(&secrets_path)?;
    let redaction = read_json(&redaction_path)?;
    let forbidden_patterns = read_json(&forbidden_patterns_path)?;
    let dependency_policy = read_json(&dependency_policy_path)?;
    let signing_policy = read_yaml(&signing_policy_path)?;

    let asset_rows = assets
        .get("assets")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let threat_rows = threats
        .get("threats")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mitigation_rows = mitigations
        .get("mitigations")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let control_rows = controls
        .get("controls")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let principal_rows = principals
        .get("principals")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let action_rows = actions
        .get("actions")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let resource_rows = resources
        .get("resources")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let policy_rows = policy
        .get("rules")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let auth_default_stance = auth_model
        .get("default_stance")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    let auth_support = auth_model
        .get("auth_support")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    let auth_methods = auth_model
        .get("supported_methods")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let auth_env = auth_model
        .get("runtime_auth_mode_env")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    let auth_docs_model = auth_model
        .get("docs")
        .and_then(|value| value.get("model"))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    let auth_docs_runbook = auth_model
        .get("docs")
        .and_then(|value| value.get("runbook"))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default();
    let sec_auth_001 = matches!(auth_default_stance, "public" | "internal" | "zero-trust")
        && matches!(auth_support, "no-auth" | "supported")
        && !auth_methods.is_empty()
        && !auth_env.is_empty()
        && auth_env == "ATLAS_AUTH_MODE"
        && root.join(auth_docs_model).exists()
        && root.join(auth_docs_runbook).exists()
        && !principal_rows.is_empty()
        && !action_rows.is_empty()
        && !resource_rows.is_empty();
    let principal_ids = principal_rows
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let action_ids = action_rows
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let resource_ids = resource_rows
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let mut auth_policy_unknowns = Vec::new();
    for row in &policy_rows {
        let rule_id = row
            .get("id")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or("unknown-rule");
        for principal in row
            .get("principals")
            .and_then(serde_yaml::Value::as_sequence)
            .into_iter()
            .flatten()
            .filter_map(serde_yaml::Value::as_str)
        {
            if !principal_ids.contains(principal) {
                auth_policy_unknowns.push(format!("{rule_id}:principal:{principal}"));
            }
        }
        for action in row
            .get("actions")
            .and_then(serde_yaml::Value::as_sequence)
            .into_iter()
            .flatten()
            .filter_map(serde_yaml::Value::as_str)
        {
            if !action_ids.contains(action) {
                auth_policy_unknowns.push(format!("{rule_id}:action:{action}"));
            }
        }
        for kind in row
            .get("resources")
            .and_then(|value| value.get("kinds"))
            .and_then(serde_yaml::Value::as_sequence)
            .into_iter()
            .flatten()
            .filter_map(serde_yaml::Value::as_str)
        {
            if !resource_ids.contains(kind) {
                auth_policy_unknowns.push(format!("{rule_id}:resource:{kind}"));
            }
        }
    }
    let sec_auth_002 = !policy_rows.is_empty() && auth_policy_unknowns.is_empty();
    let main_source = fs::read_to_string(root.join("crates/bijux-atlas-server/src/main.rs"))
        .map_err(|err| format!("failed to read runtime main source: {err}"))?;
    let runbook_text = fs::read_to_string(root.join(auth_docs_runbook))
        .map_err(|err| format!("failed to read {}: {err}", auth_docs_runbook))?;
    let auth_supports_disabled = auth_methods
        .iter()
        .filter_map(serde_yaml::Value::as_str)
        .any(|value| value == "disabled");
    let sec_auth_003 = main_source.contains("event_id = \"auth_mode_selected\"")
        && main_source.contains("auth_disabled =");
    let runbook_text_lower = runbook_text.to_ascii_lowercase();
    let sec_auth_004 = !auth_supports_disabled
        || runbook_text_lower.contains("reverse proxy")
        || runbook_text_lower.contains("ingress auth proxy");

    let mitigation_ids = mitigation_rows
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let mut missing_mitigations = Vec::new();
    let mut missing_control_or_reason = Vec::new();
    let mut high_severity_gaps = Vec::new();
    for row in &threat_rows {
        let id = row
            .get("id")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        let severity = row
            .get("severity")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        let mapped = row
            .get("mitigations")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if mapped.is_empty() {
            missing_mitigations.push(id.to_string());
        }
        let mapped_ids = mapped
            .iter()
            .filter_map(serde_yaml::Value::as_str)
            .collect::<Vec<_>>();
        if mapped_ids.iter().any(|name| !mitigation_ids.contains(name)) {
            missing_mitigations.push(id.to_string());
        }
        if severity == "high" {
            let has_executable_or_runbook = mitigation_rows.iter().any(|mitigation| {
                let mitigation_id = mitigation
                    .get("id")
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or_default();
                mapped_ids.contains(&mitigation_id)
                    && (mitigation
                        .get("control_check_id")
                        .and_then(serde_yaml::Value::as_str)
                        .is_some()
                        || mitigation
                            .get("runbook_page")
                            .and_then(serde_yaml::Value::as_str)
                            .is_some())
            });
            if !has_executable_or_runbook {
                high_severity_gaps.push(id.to_string());
            }
        }
    }

    let mut missing_docs_links = Vec::new();
    for row in &mitigation_rows {
        let id = row
            .get("id")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        let has_control = row
            .get("control_check_id")
            .and_then(serde_yaml::Value::as_str)
            .is_some();
        let has_reason = row
            .get("documented_reason")
            .and_then(serde_yaml::Value::as_str)
            .is_some();
        if !has_control && !has_reason {
            missing_control_or_reason.push(id.to_string());
        }
        let docs_page = row
            .get("docs_page")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        if docs_page.is_empty() || !root.join(docs_page).exists() {
            missing_docs_links.push(id.to_string());
        }
    }

    let mut shape_errors = Vec::new();
    if asset_rows.is_empty() {
        shape_errors.push("assets.yaml must define at least one asset".to_string());
    }
    for row in &asset_rows {
        if row.get("id").and_then(serde_yaml::Value::as_str).is_none()
            || row
                .get("type")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("description")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("sensitivity")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("owner")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
        {
            shape_errors.push("assets.yaml contains an asset missing required fields".to_string());
            break;
        }
    }
    if threat_rows.is_empty() {
        shape_errors.push("threats.yaml must define at least one threat".to_string());
    }
    for row in &threat_rows {
        if row.get("id").and_then(serde_yaml::Value::as_str).is_none()
            || row
                .get("category")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("title")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("severity")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("likelihood")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("affected_component")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("residual_risk")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
        {
            shape_errors.push("threats.yaml contains a threat missing required fields".to_string());
            break;
        }
    }
    if mitigation_rows.is_empty() {
        shape_errors.push("mitigations.yaml must define at least one mitigation".to_string());
    }
    for row in &mitigation_rows {
        if row.get("id").and_then(serde_yaml::Value::as_str).is_none()
            || row
                .get("title")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
            || row
                .get("docs_page")
                .and_then(serde_yaml::Value::as_str)
                .is_none()
        {
            shape_errors
                .push("mitigations.yaml contains a mitigation missing required fields".to_string());
            break;
        }
    }
    if control_rows.is_empty() {
        shape_errors.push("controls.yaml must define at least one control".to_string());
    }

    let sec_threat_001 = shape_errors.is_empty();
    let sec_threat_002 = missing_mitigations.is_empty();
    let sec_threat_003 = missing_control_or_reason.is_empty() && missing_docs_links.is_empty();
    let sec_threat_004 = high_severity_gaps.is_empty();
    let declared_secret_keys = secrets
        .get("secrets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("key").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let redaction_keys = redaction
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("key").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let missing_redaction_keys = declared_secret_keys
        .iter()
        .filter(|key| !redaction_keys.contains(**key))
        .map(|key| (*key).to_string())
        .collect::<Vec<_>>();
    let default_scan_dir = root.join("release/evidence");
    let forbidden_literals = forbidden_patterns
        .get("patterns")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("literal").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let evidence_matches = scan_matches(&root, &default_scan_dir, &forbidden_literals)?;
    let sec_red_001 = missing_redaction_keys.is_empty();
    let sec_red_002 = evidence_matches.is_empty();

    let npm_allowed_registries = dependency_policy
        .get("npm")
        .and_then(|value| value.get("allowed_registries"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let npm_lockfiles = dependency_policy
        .get("npm")
        .and_then(|value| value.get("lockfile_paths"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let mut disallowed_npm_sources = Vec::new();
    for lockfile in npm_lockfiles {
        let lockfile_path = root.join(lockfile);
        let lockfile_json = read_json(&lockfile_path)?;
        let mut resolved_urls = Vec::new();
        collect_json_key_strings(&lockfile_json, "resolved", &mut resolved_urls);
        for url in resolved_urls {
            if !npm_allowed_registries
                .iter()
                .any(|prefix| url.starts_with(prefix))
            {
                disallowed_npm_sources.push(format!("{lockfile}:{url}"));
            }
        }
    }

    let python_allowed_indexes = dependency_policy
        .get("python")
        .and_then(|value| value.get("allowed_indexes"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let python_requirement_paths = dependency_policy
        .get("python")
        .and_then(|value| value.get("requirements_paths"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let mut explicit_python_indexes = Vec::new();
    let mut disallowed_python_indexes = Vec::new();
    for requirement_path in python_requirement_paths {
        let abs_path = root.join(requirement_path);
        let text = fs::read_to_string(&abs_path)
            .map_err(|err| format!("failed to read {}: {err}", abs_path.display()))?;
        for index in parse_requirement_indexes(&text) {
            explicit_python_indexes.push(format!("{requirement_path}:{index}"));
            if !python_allowed_indexes
                .iter()
                .any(|allowed| allowed == &index)
            {
                disallowed_python_indexes.push(format!("{requirement_path}:{index}"));
            }
        }
    }
    let python_default_allowed = python_allowed_indexes
        .iter()
        .any(|index| *index == "https://pypi.org/simple" || *index == "https://pypi.org/simple/");
    if explicit_python_indexes.is_empty() && !python_default_allowed {
        disallowed_python_indexes.push("python-default-index:not-allowlisted".to_string());
    }

    let workflow_dir = dependency_policy
        .get("github_actions")
        .and_then(|value| value.get("workflow_dir"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(".github/workflows");
    let action_inventory_path = dependency_policy
        .get("github_actions")
        .and_then(|value| value.get("toolchain_inventory"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("ops/inventory/toolchain.json");
    let action_exceptions_path = dependency_policy
        .get("github_actions")
        .and_then(|value| value.get("exceptions_path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("configs/security/github-actions-exceptions.json");
    let action_inventory = read_json(&root.join(action_inventory_path))?;
    let action_exceptions = read_json(&root.join(action_exceptions_path))?;
    let allowed_actions = action_inventory
        .get("github_actions")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    let exception_rows = action_exceptions
        .get("exceptions")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut invalid_action_exceptions = Vec::new();
    for row in &exception_rows {
        let workflow_path = row
            .get("workflow_path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let action = row
            .get("action")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let reason = row
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let owner = row
            .get("owner")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let expires_on = row
            .get("expires_on")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if workflow_path.is_empty()
            || action.is_empty()
            || reason.is_empty()
            || owner.is_empty()
            || !is_iso_date(expires_on)
        {
            invalid_action_exceptions.push(format!(
                "{workflow_path}:{action}:owner-or-expiry-invalid"
            ));
        }
    }
    let mut workflow_pin_gaps = Vec::new();
    let mut workflow_action_rows = Vec::new();
    for file in list_files_recursive(&root.join(workflow_dir))? {
        if file.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(&file)
            .display()
            .to_string();
        let text = fs::read_to_string(&file)
            .map_err(|err| format!("failed to read {}: {err}", file.display()))?;
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
            let Some((action_name, reference)) = spec.rsplit_once('@') else {
                workflow_action_rows.push(serde_json::json!({
                    "workflow_path": rel.clone(),
                    "line": line_idx + 1,
                    "action": spec,
                    "reference": serde_json::Value::Null,
                    "status": "missing-ref"
                }));
                workflow_pin_gaps.push(format!("{rel}:{}:{spec}:missing-ref", line_idx + 1));
                continue;
            };
            let exception = exception_rows.iter().find(|row| {
                row.get("workflow_path")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| value == rel.as_str())
                    && row
                        .get("action")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value == action_name)
                    && row
                        .get("reason")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| !value.is_empty())
                    && row
                        .get("owner")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| !value.is_empty())
                    && row
                        .get("expires_on")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(is_iso_date)
            });
            if !is_full_sha(reference) {
                let allow_exception = exception.is_some();
                workflow_action_rows.push(serde_json::json!({
                    "workflow_path": rel.clone(),
                    "line": line_idx + 1,
                    "action": action_name,
                    "reference": reference,
                    "status": if allow_exception { "exception" } else { "unpinned" }
                }));
                if allow_exception {
                    continue;
                }
                workflow_pin_gaps.push(format!("{rel}:{}:{action_name}:{reference}", line_idx + 1));
                continue;
            }
            let Some(entry) = allowed_actions.get(action_name) else {
                workflow_action_rows.push(serde_json::json!({
                    "workflow_path": rel.clone(),
                    "line": line_idx + 1,
                    "action": action_name,
                    "reference": reference,
                    "status": "not-allowlisted"
                }));
                workflow_pin_gaps.push(format!(
                    "{rel}:{}:{action_name}:not-allowlisted",
                    line_idx + 1
                ));
                continue;
            };
            let expected_sha = entry
                .get("sha")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if expected_sha != reference {
                workflow_action_rows.push(serde_json::json!({
                    "workflow_path": rel.clone(),
                    "line": line_idx + 1,
                    "action": action_name,
                    "reference": reference,
                    "status": "inventory-mismatch",
                    "expected_sha": expected_sha
                }));
                workflow_pin_gaps.push(format!(
                    "{rel}:{}:{action_name}:expected-{expected_sha}-got-{reference}",
                    line_idx + 1
                ));
            } else {
                workflow_action_rows.push(serde_json::json!({
                    "workflow_path": rel.clone(),
                    "line": line_idx + 1,
                    "action": action_name,
                    "reference": reference,
                    "status": "pinned"
                }));
            }
        }
    }

    let image_policy = dependency_policy
        .get("images")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let image_inventory_path = image_policy
        .get("toolchain_inventory")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("ops/inventory/toolchain.json");
    let bases_lock_path = image_policy
        .get("bases_lock")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("docker/bases.lock");
    let evidence_manifest_path = image_policy
        .get("evidence_manifest")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("release/evidence/manifest.json");
    let image_inventory = read_json(&root.join(image_inventory_path))?;
    let bases_lock = read_json(&root.join(bases_lock_path))?;
    let evidence_manifest = read_json(&root.join(evidence_manifest_path))?;
    let mut image_evidence_gaps = Vec::new();
    if let Some(images) = image_inventory
        .get("images")
        .and_then(serde_json::Value::as_object)
    {
        for (name, image_ref) in images {
            if name == "generated_by" {
                continue;
            }
            let image_ref = image_ref.as_str().unwrap_or_default();
            if !image_ref.contains("@sha256:") {
                image_evidence_gaps.push(format!("toolchain:{name}"));
            }
        }
    }
    if let Some(images) = bases_lock
        .get("images")
        .and_then(serde_json::Value::as_array)
    {
        for image in images {
            let name = image
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let digest = image
                .get("digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if !is_sha256_digest(digest) {
                image_evidence_gaps.push(format!("bases-lock:{name}"));
            }
        }
    } else {
        image_evidence_gaps.push("bases-lock:missing-images".to_string());
    }
    let manifest_bases_path = evidence_manifest
        .get("docker_bases_lock")
        .and_then(|value| value.get("path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if manifest_bases_path != bases_lock_path {
        image_evidence_gaps.push(format!(
            "evidence-manifest:bases-lock:{manifest_bases_path}"
        ));
    }
    let sec_images_001 = image_evidence_gaps.is_empty();

    let sbom_policy = dependency_policy
        .get("sbom")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let required_sbom_formats = sbom_policy
        .get("required_formats")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let sbom_rows = evidence_manifest
        .get("sboms")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let image_rows = evidence_manifest
        .get("image_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let prod_digests = image_rows
        .iter()
        .filter(|row| {
            row.get("profile")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|profile| profile.starts_with("prod"))
        })
        .filter_map(|row| row.get("digest").and_then(serde_json::Value::as_str))
        .filter(|digest| !digest.is_empty())
        .collect::<Vec<_>>();
    let mut sbom_gaps = Vec::new();
    if sbom_rows.is_empty() {
        sbom_gaps.push("sboms:missing".to_string());
    }
    for row in &sbom_rows {
        let format = row
            .get("format")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if !required_sbom_formats
            .iter()
            .any(|allowed| allowed == &format)
        {
            sbom_gaps.push(format!("format:{format}"));
        }
    }
    for digest in &prod_digests {
        let has_match = sbom_rows.iter().any(|row| {
            row.get("image_ref")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|image_ref| image_ref == *digest || image_ref.ends_with(digest))
        });
        if !has_match {
            sbom_gaps.push(format!("prod-digest:{digest}"));
        }
    }
    let sec_sbom_001 = sbom_gaps.is_empty();

    let signing_items = signing_policy
        .get("signed_items")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut signing_policy_gaps = Vec::new();
    if signing_items.is_empty() {
        signing_policy_gaps.push("signed_items".to_string());
    }
    if signing_policy
        .get("key_custody_model")
        .and_then(|value| value.get("owner"))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default()
        .is_empty()
    {
        signing_policy_gaps.push("key_custody_model.owner".to_string());
    }
    if signing_policy
        .get("verification")
        .and_then(|value| value.get("command"))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default()
        .is_empty()
    {
        signing_policy_gaps.push("verification.command".to_string());
    }
    for item in &signing_items {
        let path = item
            .get("path")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        if path.is_empty() || !root.join(path).exists() {
            signing_policy_gaps.push(format!("missing-signed-item:{path}"));
        }
    }
    let signing_policy_valid = signing_policy_gaps.is_empty();
    let sec_deps_001 = disallowed_npm_sources.is_empty();
    let sec_deps_002 = disallowed_python_indexes.is_empty();
    if !invalid_action_exceptions.is_empty() {
        workflow_pin_gaps.extend(invalid_action_exceptions.clone());
    }
    let sec_actions_001 = workflow_pin_gaps.is_empty();
    let github_actions_report = serde_json::json!({
        "schema_version": 1,
        "status": if sec_actions_001 { "ok" } else { "failed" },
        "workflow_dir": workflow_dir,
        "inventory_path": action_inventory_path,
        "exceptions_path": action_exceptions_path,
        "summary": {
            "total_refs": workflow_action_rows.len(),
            "unpinned_or_invalid": workflow_pin_gaps.len(),
            "exceptions": exception_rows.len()
        },
        "rows": workflow_action_rows
    });
    let github_actions_report_path = named_report_path(&root, "security-github-actions.json")?;
    fs::write(
        &github_actions_report_path,
        serde_json::to_string_pretty(&github_actions_report)
            .map_err(|err| format!("encode github actions report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", github_actions_report_path.display()))?;

    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if sec_threat_001
            && sec_threat_002
            && sec_threat_003
            && sec_threat_004
            && sec_red_001
            && sec_red_002
            && sec_auth_002
            && sec_auth_003
            && sec_auth_004
            && sec_deps_001
            && sec_deps_002
            && sec_images_001
            && sec_actions_001
            && sec_sbom_001
            && signing_policy_valid
        { "ok" } else { "failed" },
        "counts": {
            "assets": asset_rows.len(),
            "threats": threat_rows.len(),
            "mitigations": mitigation_rows.len(),
            "controls": control_rows.len(),
            "principals": principal_rows.len(),
            "actions": action_rows.len(),
            "resources": resource_rows.len(),
            "auth_policy_rules": policy_rows.len(),
            "declared_secrets": declared_secret_keys.len(),
            "prod_image_digests": prod_digests.len(),
            "sboms": sbom_rows.len(),
            "signed_items": signing_items.len()
        },
        "reports": {
            "github_actions": github_actions_report_path
                .strip_prefix(&root)
                .unwrap_or(&github_actions_report_path)
                .display()
                .to_string()
        },
        "contracts": {
            "SEC-THREAT-001": sec_threat_001,
            "SEC-THREAT-002": sec_threat_002,
            "SEC-THREAT-003": sec_threat_003,
            "SEC-THREAT-004": sec_threat_004,
            "SEC-AUTH-001": sec_auth_001,
            "SEC-AUTH-002": sec_auth_002,
            "SEC-AUTH-003": sec_auth_003,
            "SEC-AUTH-004": sec_auth_004,
            "SEC-RED-001": sec_red_001,
            "SEC-RED-002": sec_red_002,
            "SEC-DEPS-001": sec_deps_001,
            "SEC-DEPS-002": sec_deps_002,
            "SEC-IMAGES-001": sec_images_001,
            "SEC-ACTIONS-001": sec_actions_001,
            "SEC-SBOM-001": sec_sbom_001
        },
        "policy_validation": {
            "dependency_source_policy": sec_deps_001 && sec_deps_002 && sec_images_001 && sec_actions_001 && sec_sbom_001,
            "signing_policy": signing_policy_valid
        },
        "gaps": {
            "shape_errors": shape_errors,
            "auth_model_gaps": if sec_auth_001 {
                Vec::<String>::new()
            } else {
                vec!["auth model, supporting registries, or linked docs are incomplete".to_string()]
            },
            "auth_policy_unknowns": auth_policy_unknowns,
            "missing_mitigations": missing_mitigations,
            "missing_control_or_reason": missing_control_or_reason,
            "missing_docs_links": missing_docs_links,
            "high_severity_gaps": high_severity_gaps,
            "missing_redaction_keys": missing_redaction_keys,
            "evidence_secret_matches": evidence_matches,
            "disallowed_npm_sources": disallowed_npm_sources,
            "disallowed_python_indexes": disallowed_python_indexes,
            "workflow_pin_gaps": workflow_pin_gaps,
            "invalid_action_exceptions": invalid_action_exceptions,
            "image_evidence_gaps": image_evidence_gaps,
            "sbom_gaps": sbom_gaps,
            "signing_policy_gaps": signing_policy_gaps
        }
    });

    let path = report_path(&root)?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode security report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": payload["status"].clone(),
            "text": if payload["status"] == "ok" { "security threat model validated" } else { "security threat model validation failed" },
            "rows": [{
                "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
                "contracts": payload["contracts"].clone(),
                "gaps": payload["gaps"].clone()
            }],
            "summary": {"total": 1, "errors": if payload["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 1 }))
}

fn run_security_compliance_validate(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let controls_schema_path = root.join("configs/contracts/security/controls.schema.json");
    let matrix_schema_path = root.join("configs/contracts/security/compliance-matrix.schema.json");
    ensure_json(&controls_schema_path)?;
    ensure_json(&matrix_schema_path)?;
    let controls = read_yaml(&root.join("security/compliance/controls.yaml"))?;
    let matrix = read_yaml(&root.join("security/compliance/matrix.yaml"))?;
    let control_rows = controls
        .get("controls")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let matrix_rows = matrix
        .get("mappings")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut evidence_gaps = Vec::new();
    let mut missing_files = Vec::new();
    for control in &control_rows {
        let id = control
            .get("id")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        let mapping = matrix_rows
            .iter()
            .find(|row| row.get("control_id").and_then(serde_yaml::Value::as_str) == Some(id));
        let Some(mapping) = mapping else {
            evidence_gaps.push(id.to_string());
            continue;
        };
        let evidence = mapping
            .get("evidence")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        if evidence.is_empty() {
            evidence_gaps.push(id.to_string());
        }
        for item in evidence {
            if let Some(path) = item.as_str() {
                if !root.join(path).exists() {
                    missing_files.push(format!("{id}:{path}"));
                }
            }
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if evidence_gaps.is_empty() && missing_files.is_empty() { "ok" } else { "failed" },
        "contracts": {
            "SEC-COMP-001": true,
            "SEC-COMP-002": evidence_gaps.is_empty(),
            "SEC-COMP-003": missing_files.is_empty()
        },
        "gaps": {
            "controls_without_evidence": evidence_gaps,
            "missing_evidence_files": missing_files
        },
        "counts": {
            "controls": control_rows.len(),
            "mappings": matrix_rows.len()
        }
    });
    let path = named_report_path(&root, "security-compliance.json")?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode security compliance report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": payload["status"].clone(),
            "text": if payload["status"] == "ok" { "security compliance validated" } else { "security compliance validation failed" },
            "rows": [{
                "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
                "contracts": payload["contracts"].clone(),
                "gaps": payload["gaps"].clone()
            }],
            "summary": {"total": 1, "errors": if payload["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 1 }))
}

fn run_security_scan_artifacts(args: SecurityScanArtifactsArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let schema_path = root.join("configs/contracts/security/forbidden-patterns.schema.json");
    let policy_path = root.join("configs/security/forbidden-patterns.json");
    ensure_json(&schema_path)?;
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&policy_path)
            .map_err(|err| format!("failed to read {}: {err}", policy_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", policy_path.display()))?;
    let patterns = policy
        .get("patterns")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("literal").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let dir = if args.dir.is_absolute() {
        args.dir
    } else {
        root.join(args.dir)
    };
    let matches = scan_matches(&root, &dir, &patterns)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if matches.is_empty() { "ok" } else { "failed" },
        "contracts": {
            "SEC-ART-001": matches.is_empty()
        },
        "scan_root": dir.strip_prefix(&root).unwrap_or(&dir).display().to_string(),
        "matches": matches
    });
    let path = named_report_path(&root, "security-artifact-scan.json")?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode artifact scan report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": payload["status"].clone(),
            "text": if payload["status"] == "ok" { "artifact scan passed" } else { "artifact scan found forbidden patterns" },
            "rows": [{
                "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
                "scan_root": payload["scan_root"].clone(),
                "matches": payload["matches"].clone()
            }],
            "summary": {"total": 1, "errors": if payload["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 1 }))
}
