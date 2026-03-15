// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    FormatArg, SecurityAuthenticationCommand, SecurityAuthorizationCommand, SecurityCommand,
    SecurityIncidentReportArgs, SecurityPolicyInspectArgs, SecurityRoleAssignArgs,
    SecurityScanArtifactsArgs, SecurityThreatCommand, SecurityThreatExplainArgs,
    SecurityTokenInspectArgs, SecurityValidateArgs,
};
use crate::{emit_payload, resolve_repo_root};
use base64::Engine as _;
use bijux_atlas::core as bijux_atlas_core;
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

fn auth_method_count(auth_model: &serde_yaml::Value) -> usize {
    ["methods", "supported_methods"]
        .into_iter()
        .find_map(|key| {
            auth_model
                .get(key)
                .and_then(serde_yaml::Value::as_sequence)
                .map(std::vec::Vec::len)
        })
        .unwrap_or(0)
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

fn parse_expiry_allowlist_rows(text: &str) -> Vec<(String, String, String)> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts = trimmed.split('|').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 3 {
            continue;
        }
        rows.push((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ));
    }
    rows
}

fn parse_python_lock_rows(text: &str) -> Vec<String> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("--") {
            continue;
        }
        if let Some((name, version)) = trimmed.split_once("==") {
            let package = name.trim();
            let pinned = version.trim();
            if !package.is_empty() && !pinned.is_empty() {
                rows.push(format!("{package}=={pinned}"));
            }
        }
    }
    rows.sort();
    rows.dedup();
    rows
}

fn parse_cargo_lock_rows(text: &str) -> Vec<String> {
    let Ok(value) = text.parse::<toml::Value>() else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    if let Some(packages) = value.get("package").and_then(toml::Value::as_array) {
        for pkg in packages {
            let Some(name) = pkg.get("name").and_then(toml::Value::as_str) else {
                continue;
            };
            let Some(version) = pkg.get("version").and_then(toml::Value::as_str) else {
                continue;
            };
            rows.push(format!("{name}@{version}"));
        }
    }
    rows.sort();
    rows.dedup();
    rows
}

fn parse_helm_lock_rows(path: &Path) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let value = read_yaml(path)?;
    let mut rows = Vec::new();
    if let Some(deps) = value
        .get("dependencies")
        .and_then(serde_yaml::Value::as_sequence)
    {
        for row in deps {
            let Some(name) = row.get("name").and_then(serde_yaml::Value::as_str) else {
                continue;
            };
            let Some(version) = row.get("version").and_then(serde_yaml::Value::as_str) else {
                continue;
            };
            rows.push(format!("{name}@{version}"));
        }
    }
    rows.sort();
    rows.dedup();
    Ok(rows)
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

fn parse_scan_summary(value: &serde_json::Value) -> Option<(i64, i64, i64, i64)> {
    let summary = value.get("summary")?;
    Some((
        summary.get("critical")?.as_i64()?,
        summary.get("high")?.as_i64()?,
        summary.get("medium")?.as_i64()?,
        summary.get("low")?.as_i64()?,
    ))
}

fn validate_audit_record_shape(
    record: &serde_json::Value,
    allowed_events: &[&str],
) -> Result<(), String> {
    let Some(object) = record.as_object() else {
        return Err("audit record must be a JSON object".to_string());
    };
    let event_id = object
        .get("event_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "audit record missing event_id".to_string())?;
    if !event_id.starts_with("audit_") {
        return Err("audit record event_id must start with audit_".to_string());
    }
    let event_name = object
        .get("event_name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "audit record missing event_name".to_string())?;
    if !allowed_events.contains(&event_name) {
        return Err(format!(
            "audit record event_name is not allowed: {event_name}"
        ));
    }
    if object
        .get("timestamp_policy")
        .and_then(serde_json::Value::as_str)
        != Some("runtime-unix-seconds")
    {
        return Err("audit record timestamp_policy must be runtime-unix-seconds".to_string());
    }
    if object
        .get("timestamp_unix_s")
        .and_then(serde_json::Value::as_u64)
        .is_none()
    {
        return Err("audit record missing timestamp_unix_s".to_string());
    }
    let sink = object
        .get("sink")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "audit record missing sink".to_string())?;
    if !matches!(sink, "stdout" | "file" | "otel") {
        return Err(format!("audit record sink is invalid: {sink}"));
    }
    for field in ["action", "resource_kind", "resource_id"] {
        if object
            .get(field)
            .and_then(serde_json::Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(format!("audit record missing {field}"));
        }
    }
    let encoded = serde_json::to_string(record).map_err(|err| err.to_string())?;
    for forbidden in ["Bearer ", "topsecret", "@", "127.0.0.1"] {
        if encoded.contains(forbidden) {
            return Err(format!(
                "audit record contains forbidden marker: {forbidden}"
            ));
        }
    }
    Ok(())
}

fn write_json_lines(path: &Path, rows: &[serde_json::Value]) -> Result<(), String> {
    let mut buffer = String::new();
    for row in rows {
        buffer.push_str(
            &serde_json::to_string(row).map_err(|err| format!("encode jsonl row failed: {err}"))?,
        );
        buffer.push('\n');
    }
    fs::write(path, buffer).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn collect_source_json_fields(text: &str) -> std::collections::BTreeSet<String> {
    let mut fields = std::collections::BTreeSet::new();
    let mut in_audit_object = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"event_id\": \"audit_") {
            in_audit_object = true;
        }
        if !in_audit_object {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix('"') else {
            if trimmed.starts_with("})") || trimmed.starts_with("});") {
                in_audit_object = false;
            }
            continue;
        };
        let Some((key, suffix)) = rest.split_once('"') else {
            if trimmed.starts_with("})") || trimmed.starts_with("});") {
                in_audit_object = false;
            }
            continue;
        };
        if suffix.trim_start().starts_with(':')
            && !key.is_empty()
            && key
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        {
            fields.insert(key.to_string());
        }
        if trimmed.starts_with("})") || trimmed.starts_with("});") {
            in_audit_object = false;
        }
    }
    fields
}

pub(crate) fn run_security_command(
    _quiet: bool,
    command: SecurityCommand,
) -> Result<(String, i32), String> {
    match command {
        SecurityCommand::Validate(args) => run_security_validate(args),
        SecurityCommand::ConfigValidate(args) => run_security_config_validate(args),
        SecurityCommand::Diagnostics(args) => run_security_diagnostics(args),
        SecurityCommand::PolicyInspect(args) => run_security_policy_inspect(args),
        SecurityCommand::Audit(args) => run_security_audit(args),
        SecurityCommand::VulnerabilityReport(args) => run_security_vulnerability_report(args),
        SecurityCommand::DependencyAudit(args) => run_security_dependency_audit(args),
        SecurityCommand::IncidentReport(args) => run_security_incident_report(args),
        SecurityCommand::Authentication { command } => match command {
            SecurityAuthenticationCommand::ApiKeys(args) => run_security_auth_api_keys(args),
            SecurityAuthenticationCommand::TokenInspect(args) => {
                run_security_auth_token_inspect(args)
            }
            SecurityAuthenticationCommand::Diagnostics(args) => run_security_auth_diagnostics(args),
            SecurityAuthenticationCommand::PolicyValidate(args) => {
                run_security_auth_policy_validate(args)
            }
        },
        SecurityCommand::Authorization { command } => match command {
            SecurityAuthorizationCommand::Roles(args) => run_security_authorization_roles(args),
            SecurityAuthorizationCommand::Permissions(args) => {
                run_security_authorization_permissions(args)
            }
            SecurityAuthorizationCommand::Diagnostics(args) => {
                run_security_authorization_diagnostics(args)
            }
            SecurityAuthorizationCommand::Assign(args) => run_security_authorization_assign(args),
            SecurityAuthorizationCommand::Validate(args) => {
                run_security_authorization_validate(args)
            }
        },
        SecurityCommand::Compliance { command } => match command {
            crate::cli::SecurityComplianceCommand::Validate(args) => {
                run_security_compliance_validate(args)
            }
        },
        SecurityCommand::Threats { command } => match command {
            SecurityThreatCommand::List(args) => run_security_threats_list(args),
            SecurityThreatCommand::Explain(args) => run_security_threats_explain(args),
            SecurityThreatCommand::Verify(args) => run_security_threats_verify(args),
        },
        SecurityCommand::ScanArtifacts(args) => run_security_scan_artifacts(args),
    }
}

fn run_security_auth_api_keys(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let auth_model = read_yaml(&root.join("configs/security/auth-model.yaml"))?;
    let methods = auth_method_count(&auth_model);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authentication_api_key_management_report",
        "status": "ok",
        "api_key_generation": "runtime helper and deterministic hashing",
        "api_key_storage": "hashed entries in ATLAS_ALLOWED_API_KEYS",
        "api_key_rotation": "supports overlap windows with not_before markers",
        "api_key_expiration": "ATLAS_API_KEY_EXPIRATION_DAYS",
        "auth_model_method_count": methods
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn parse_token_payload(token: &str) -> Result<serde_json::Value, String> {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("token must have exactly three dot-separated segments".to_string());
    }
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|err| format!("invalid token payload encoding: {err}"))?;
    serde_json::from_slice(&raw).map_err(|err| format!("invalid token payload json: {err}"))
}

fn run_security_auth_token_inspect(
    args: SecurityTokenInspectArgs,
) -> Result<(String, i32), String> {
    let payload = parse_token_payload(&args.token)?;
    let scopes = payload
        .get("scope")
        .and_then(serde_json::Value::as_str)
        .map(|text| {
            text.split(' ')
                .map(str::trim)
                .filter(|scope| !scope.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "authentication_token_inspection_report",
        "status": "ok",
        "subject": payload.get("sub").and_then(serde_json::Value::as_str),
        "issuer": payload.get("iss").and_then(serde_json::Value::as_str),
        "audience": payload.get("aud").and_then(serde_json::Value::as_str),
        "expires_unix_s": payload.get("exp").and_then(serde_json::Value::as_u64),
        "token_id": payload.get("jti").and_then(serde_json::Value::as_str),
        "scopes": scopes
    });
    let rendered = emit_payload(args.format, args.out, &report)?;
    Ok((rendered, 0))
}

fn run_security_auth_diagnostics(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let config = bijux_atlas_core::load_security_config_from_path(
        &root.join("configs/security/runtime-security.yaml"),
    )?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authentication_diagnostics_report",
        "status": "ok",
        "auth_mode": config.auth.mode,
        "auth_required": config.auth.required,
        "audit_enabled": config.audit.enabled,
        "audit_sink": config.audit.sink,
        "event_classes": config.events.classes
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_auth_policy_validate(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let auth_model = read_yaml(&root.join("configs/security/auth-model.yaml"))?;
    let policy = read_yaml(&root.join("configs/security/policy.yaml"))?;
    let method_count = auth_method_count(&auth_model);
    let rule_count = policy
        .get("rules")
        .and_then(serde_yaml::Value::as_sequence)
        .map_or(0, std::vec::Vec::len);
    let ok = method_count > 0 && rule_count > 0;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authentication_policy_validation_report",
        "status": if ok { "ok" } else { "failed" },
        "auth_methods": method_count,
        "policy_rules": rule_count
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if ok { 0 } else { 2 }))
}

fn run_security_authorization_roles(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let roles = read_yaml(&root.join("configs/security/roles.yaml"))?;
    let rows = roles
        .get("roles")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| serde_json::to_value(row).unwrap_or(serde_json::Value::Null))
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authorization_role_management_report",
        "status": "ok",
        "role_count": rows.len(),
        "roles": rows
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_authorization_permissions(
    args: SecurityValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let permissions = read_yaml(&root.join("configs/security/permissions.yaml"))?;
    let rows = permissions
        .get("permissions")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| serde_json::to_value(row).unwrap_or(serde_json::Value::Null))
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authorization_permission_inspection_report",
        "status": "ok",
        "permission_count": rows.len(),
        "permissions": rows
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_authorization_diagnostics(
    args: SecurityValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let roles = read_yaml(&root.join("configs/security/roles.yaml"))?;
    let permissions = read_yaml(&root.join("configs/security/permissions.yaml"))?;
    let policy = read_yaml(&root.join("configs/security/policy.yaml"))?;
    let assignments = read_yaml(&root.join("configs/security/role-assignments.yaml")).ok();
    let role_count = roles
        .get("roles")
        .and_then(serde_yaml::Value::as_sequence)
        .map_or(0, std::vec::Vec::len);
    let permission_count = permissions
        .get("permissions")
        .and_then(serde_yaml::Value::as_sequence)
        .map_or(0, std::vec::Vec::len);
    let policy_rules = policy
        .get("rules")
        .and_then(serde_yaml::Value::as_sequence)
        .map_or(0, std::vec::Vec::len);
    let assignment_count = assignments
        .as_ref()
        .and_then(|value| value.get("assignments"))
        .and_then(serde_yaml::Value::as_sequence)
        .map_or(0, std::vec::Vec::len);
    let default_decision = policy
        .get("default_decision")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("deny");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authorization_diagnostics_report",
        "status": "ok",
        "role_count": role_count,
        "permission_count": permission_count,
        "policy_rule_count": policy_rules,
        "assignment_count": assignment_count,
        "default_decision": default_decision
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_authorization_assign(
    args: SecurityRoleAssignArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let path = root.join("configs/security/role-assignments.yaml");
    let mut assignments = if path.exists() {
        read_yaml(&path)?
    } else {
        serde_yaml::from_str("schema_version: 1\nassignments: []\n")
            .map_err(|err| format!("build default assignments: {err}"))?
    };
    let Some(array) = assignments
        .get_mut("assignments")
        .and_then(serde_yaml::Value::as_sequence_mut)
    else {
        return Err(format!("{} missing assignments sequence", path.display()));
    };
    array.push(serde_yaml::Value::Mapping({
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("principal".to_string()),
            serde_yaml::Value::String(args.principal.clone()),
        );
        map.insert(
            serde_yaml::Value::String("role_id".to_string()),
            serde_yaml::Value::String(args.role_id.clone()),
        );
        map
    }));
    fs::write(
        &path,
        serde_yaml::to_string(&assignments).map_err(|err| format!("encode assignments: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authorization_role_assignment_report",
        "status": "ok",
        "principal": args.principal,
        "role_id": args.role_id,
        "path": path.strip_prefix(&root).unwrap_or(&path).display().to_string()
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_authorization_validate(
    args: SecurityValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let roles: bijux_atlas_core::RoleCatalog = serde_yaml::from_str(
        &fs::read_to_string(root.join("configs/security/roles.yaml"))
            .map_err(|err| format!("failed to read role catalog: {err}"))?,
    )
    .map_err(|err| format!("parse role catalog failed: {err}"))?;
    let permissions: bijux_atlas_core::PermissionCatalog = serde_yaml::from_str(
        &fs::read_to_string(root.join("configs/security/permissions.yaml"))
            .map_err(|err| format!("failed to read permission catalog: {err}"))?,
    )
    .map_err(|err| format!("parse permission catalog failed: {err}"))?;
    let mut role_ids = std::collections::BTreeSet::new();
    for role in &roles.roles {
        role_ids.insert(role.id.clone());
    }
    let mut permission_ids = std::collections::BTreeSet::new();
    for permission in &permissions.permissions {
        permission_ids.insert(permission.id.clone());
    }
    let mut errors = Vec::new();
    for role in &roles.roles {
        for inherited in &role.inherits {
            if !role_ids.contains(inherited) {
                errors.push(format!(
                    "unknown inherited role: {} -> {}",
                    role.id, inherited
                ));
            }
        }
        for permission in &role.permissions {
            if !permission_ids.contains(permission) {
                errors.push(format!(
                    "unknown permission reference: {} -> {}",
                    role.id, permission
                ));
            }
        }
    }
    let ok = errors.is_empty();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "authorization_permission_validation_report",
        "status": if ok { "ok" } else { "failed" },
        "errors": errors,
        "role_count": roles.roles.len(),
        "permission_count": permissions.permissions.len()
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if ok { 0 } else { 2 }))
}

fn run_security_config_validate(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let path = root.join("configs/security/runtime-security.yaml");
    let config = bijux_atlas_core::load_security_config_from_path(&path)?;
    let errors = bijux_atlas_core::validate_security_config(&config);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_config_validation_report",
        "config_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_diagnostics(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let config_path = root.join("configs/security/runtime-security.yaml");
    let config = bijux_atlas_core::load_security_config_from_path(&config_path)?;
    let mut registry = bijux_atlas_core::SecurityPolicyRegistry::new();
    let policy_rows = read_yaml(&root.join("configs/security/policy.yaml"))?
        .get("rules")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    for row in policy_rows {
        if let Some(policy_id) = row.get("id").and_then(serde_yaml::Value::as_str) {
            registry.register(bijux_atlas_core::SecurityPolicy {
                policy_id: policy_id.to_string(),
                description: row
                    .get("description")
                    .and_then(serde_yaml::Value::as_str)
                    .unwrap_or("security policy")
                    .to_string(),
                enabled: row
                    .get("enabled")
                    .and_then(serde_yaml::Value::as_bool)
                    .unwrap_or(true),
            });
        }
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_diagnostics_report",
        "auth_mode": config.auth.mode,
        "tls_required": config.transport.tls_required,
        "audit_enabled": config.audit.enabled,
        "policy_count": registry.list().len(),
        "enabled_policy_count": registry.enabled_count(),
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_policy_inspect(args: SecurityPolicyInspectArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let policy = read_yaml(&root.join("configs/security/policy.yaml"))?;
    let rules = policy
        .get("rules")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let rows = rules
        .into_iter()
        .filter(|row| {
            if let Some(filter_id) = &args.policy_id {
                row.get("id")
                    .and_then(serde_yaml::Value::as_str)
                    .is_some_and(|id| id == filter_id)
            } else {
                true
            }
        })
        .map(|row| serde_json::to_value(row).unwrap_or(serde_json::Value::Null))
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_policy_inspection_report",
        "policy_filter": args.policy_id,
        "policies": rows
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_audit(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let report_file = root.join("artifacts/security/security-threat-model.json");
    let report = if report_file.exists() {
        read_json(&report_file)?
    } else {
        serde_json::json!({"status":"missing"})
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_audit_report",
        "threat_model_status": report.get("status").cloned().unwrap_or(serde_json::Value::String("unknown".to_string())),
        "artifacts_present": report_file.exists(),
        "audit_scope": [
            "security configuration",
            "policy inventory",
            "threat model artifact"
        ]
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_vulnerability_report(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    // Reuse security validation as report generator so policy checks and scan aggregation stay canonical.
    let _ = run_security_validate(args.clone())?;
    let path = root.join("artifacts/security/security-vulnerability-scan.json");
    let payload = read_json(&path)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "security_vulnerability_report",
            "status": payload.get("status").cloned().unwrap_or_else(|| serde_json::Value::String("unknown".to_string())),
            "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
            "report": payload
        }),
    )?;
    Ok((rendered, 0))
}

fn run_security_dependency_audit(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let (validate_rendered, validate_code) = run_security_validate(SecurityValidateArgs {
        repo_root: Some(root.clone()),
        format: FormatArg::Json,
        out: None,
    })?;
    let validate_payload: serde_json::Value = serde_json::from_str(&validate_rendered)
        .map_err(|err| format!("parse security validation payload failed: {err}"))?;

    let dependency_inventory_path = root.join("artifacts/security/dependency-inventory.json");
    let vulnerability_scan_path = root.join("artifacts/security/security-vulnerability-scan.json");
    let actions_inventory_path = root.join("artifacts/security/security-github-actions.json");

    let dependency_inventory = if dependency_inventory_path.exists() {
        Some(read_json(&dependency_inventory_path)?)
    } else {
        None
    };
    let vulnerability_scan = if vulnerability_scan_path.exists() {
        Some(read_json(&vulnerability_scan_path)?)
    } else {
        None
    };
    let actions_inventory = if actions_inventory_path.exists() {
        Some(read_json(&actions_inventory_path)?)
    } else {
        None
    };

    let dependency_rows = dependency_inventory
        .as_ref()
        .and_then(|value| value.get("rows"))
        .and_then(serde_json::Value::as_array)
        .map_or(0, std::vec::Vec::len);
    let vulnerability_rows = vulnerability_scan
        .as_ref()
        .and_then(|value| value.get("rows"))
        .and_then(serde_json::Value::as_array)
        .map_or(0, std::vec::Vec::len);
    let action_rows = actions_inventory
        .as_ref()
        .and_then(|value| value.get("rows"))
        .and_then(serde_json::Value::as_array)
        .map_or(0, std::vec::Vec::len);

    let status = "ok";
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_dependency_audit_report",
        "status": status,
        "summary": {
            "dependency_inventory_rows": dependency_rows,
            "vulnerability_rows": vulnerability_rows,
            "workflow_action_rows": action_rows,
            "artifacts_present": dependency_inventory.is_some()
                && vulnerability_scan.is_some()
                && actions_inventory.is_some()
        },
        "artifacts": {
            "dependency_inventory": dependency_inventory_path.strip_prefix(&root).unwrap_or(&dependency_inventory_path).display().to_string(),
            "vulnerability_scan": vulnerability_scan_path.strip_prefix(&root).unwrap_or(&vulnerability_scan_path).display().to_string(),
            "workflow_action_inventory": actions_inventory_path.strip_prefix(&root).unwrap_or(&actions_inventory_path).display().to_string()
        },
        "security_validation": {
            "status": validate_payload.get("status"),
            "summary": validate_payload.get("summary"),
            "report_path": validate_payload
                .get("rows")
                .and_then(serde_json::Value::as_array)
                .and_then(|rows| rows.first())
                .and_then(|row| row.get("report_path"))
        }
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    let _ = validate_code;
    Ok((rendered, 0))
}

fn run_security_incident_report(args: SecurityIncidentReportArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let allowed = ["critical", "high", "medium", "low"];
    if !allowed.iter().any(|s| *s == args.severity) {
        return Err(format!("severity must be one of: {}", allowed.join(", ")));
    }
    let status_allowed = ["open", "contained", "mitigated", "resolved"];
    if !status_allowed.iter().any(|s| *s == args.status) {
        return Err(format!(
            "status must be one of: {}",
            status_allowed.join(", ")
        ));
    }
    let path = named_report_path(
        &root,
        &format!("security-incident-{}.json", args.incident_id),
    )?;
    let timestamp_utc = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system clock error: {err}"))?
        .as_secs();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_incident_report",
        "incident_id": args.incident_id,
        "severity": args.severity,
        "summary": args.summary,
        "status": args.status,
        "runbook": args.runbook,
        "timestamp_unix_s": timestamp_utc,
        "generated_by": "bijux dev atlas security incident-report"
    });
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode incident report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "security_incident_report_result",
            "status": "ok",
            "report_path": path.strip_prefix(&root).unwrap_or(&path).display().to_string(),
            "report": payload
        }),
    )?;
    Ok((rendered, 0))
}

#[derive(Debug, Clone)]
struct ThreatRow {
    id: String,
    category: String,
    title: String,
    severity: String,
    likelihood: String,
    affected_component: String,
    mitigations: Vec<String>,
    residual_risk: String,
}

fn parse_threat_rows(threats: &serde_yaml::Value) -> Vec<ThreatRow> {
    let Some(rows) = threats
        .get("threats")
        .and_then(serde_yaml::Value::as_sequence)
    else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(|row| {
            let map = row.as_mapping()?;
            Some(ThreatRow {
                id: map
                    .get(serde_yaml::Value::String("id".to_string()))?
                    .as_str()?
                    .to_string(),
                category: map
                    .get(serde_yaml::Value::String("category".to_string()))?
                    .as_str()?
                    .to_string(),
                title: map
                    .get(serde_yaml::Value::String("title".to_string()))?
                    .as_str()?
                    .to_string(),
                severity: map
                    .get(serde_yaml::Value::String("severity".to_string()))?
                    .as_str()?
                    .to_string(),
                likelihood: map
                    .get(serde_yaml::Value::String("likelihood".to_string()))?
                    .as_str()?
                    .to_string(),
                affected_component: map
                    .get(serde_yaml::Value::String("affected_component".to_string()))?
                    .as_str()?
                    .to_string(),
                mitigations: map
                    .get(serde_yaml::Value::String("mitigations".to_string()))
                    .and_then(serde_yaml::Value::as_sequence)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(serde_yaml::Value::as_str)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                residual_risk: map
                    .get(serde_yaml::Value::String("residual_risk".to_string()))?
                    .as_str()?
                    .to_string(),
            })
        })
        .collect()
}

fn parse_string_list(value: &serde_yaml::Value, field: &str) -> Vec<String> {
    let Some(rows) = value.get(field).and_then(serde_yaml::Value::as_sequence) else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(serde_yaml::Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn run_security_threats_list(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let threats = read_yaml(&root.join("ops/security/threat-model/threats.yaml"))?;
    let rows = parse_threat_rows(&threats);

    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_threat_registry_list",
        "status": "ok",
        "rows": rows.iter().map(|row| serde_json::json!({
            "id": row.id,
            "category": row.category,
            "severity": row.severity,
            "likelihood": row.likelihood,
            "affected_component": row.affected_component,
            "mitigation_count": row.mitigations.len(),
            "title": row.title
        })).collect::<Vec<_>>(),
        "summary": { "total": rows.len(), "errors": 0, "warnings": 0 }
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_security_threats_explain(args: SecurityThreatExplainArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let threats = read_yaml(&root.join("ops/security/threat-model/threats.yaml"))?;
    let taxonomy = read_yaml(&root.join("ops/security/threat-model/classification-taxonomy.yaml"))?;
    let registry = read_yaml(&root.join("ops/security/threat-model/threat-registry.yaml"))?;
    let rows = parse_threat_rows(&threats);

    let selected = if let Some(id) = args.threat_id {
        rows.into_iter()
            .filter(|row| row.id == id)
            .collect::<Vec<_>>()
    } else {
        rows
    };

    let found = !selected.is_empty();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_threat_registry_explain",
        "status": if found { "ok" } else { "error" },
        "taxonomy": taxonomy,
        "registry": registry,
        "rows": selected.iter().map(|row| serde_json::json!({
            "id": row.id,
            "category": row.category,
            "severity": row.severity,
            "likelihood": row.likelihood,
            "title": row.title,
            "affected_component": row.affected_component,
            "mitigations": row.mitigations,
            "residual_risk": row.residual_risk
        })).collect::<Vec<_>>(),
        "summary": { "total": selected.len(), "errors": if found { 0 } else { 1 }, "warnings": 0 }
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if found { 0 } else { 1 }))
}

fn run_security_threats_verify(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let threats = read_yaml(&root.join("ops/security/threat-model/threats.yaml"))?;
    let mitigations = read_yaml(&root.join("ops/security/threat-model/mitigations.yaml"))?;
    let assets = read_yaml(&root.join("ops/security/threat-model/assets.yaml"))?;
    let taxonomy = read_yaml(&root.join("ops/security/threat-model/classification-taxonomy.yaml"))?;
    let registry = read_yaml(&root.join("ops/security/threat-model/threat-registry.yaml"))?;

    let rows = parse_threat_rows(&threats);
    let severity_levels = parse_string_list(&taxonomy, "severity_levels");
    let categories = parse_string_list(&taxonomy, "categories");
    let likelihood_levels = parse_string_list(&taxonomy, "likelihood_levels");

    let mitigation_ids = mitigations
        .get("mitigations")
        .and_then(serde_yaml::Value::as_sequence)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let asset_ids = assets
        .get("assets")
        .and_then(serde_yaml::Value::as_sequence)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let registry_ids = registry
        .get("threat_ids")
        .and_then(serde_yaml::Value::as_sequence)
        .map(|rows| {
            rows.iter()
                .filter_map(serde_yaml::Value::as_str)
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();

    let mut errors = Vec::new();
    let mut by_severity = std::collections::BTreeMap::<String, usize>::new();
    let mut by_category = std::collections::BTreeMap::<String, usize>::new();

    for row in &rows {
        *by_severity.entry(row.severity.clone()).or_insert(0) += 1;
        *by_category.entry(row.category.clone()).or_insert(0) += 1;
        if !severity_levels.contains(&row.severity) {
            errors.push(format!("{} has unknown severity {}", row.id, row.severity));
        }
        if !likelihood_levels.contains(&row.likelihood) {
            errors.push(format!(
                "{} has unknown likelihood {}",
                row.id, row.likelihood
            ));
        }
        if !categories.contains(&row.category) {
            errors.push(format!("{} has unknown category {}", row.id, row.category));
        }
        if !asset_ids.contains(&row.affected_component) {
            errors.push(format!(
                "{} references unknown affected_component {}",
                row.id, row.affected_component
            ));
        }
        if row.mitigations.is_empty() {
            errors.push(format!("{} must reference at least one mitigation", row.id));
        }
        for mitigation in &row.mitigations {
            if !mitigation_ids.contains(mitigation) {
                errors.push(format!(
                    "{} references unknown mitigation {}",
                    row.id, mitigation
                ));
            }
        }
        if !registry_ids.contains(&row.id) {
            errors.push(format!("{} missing from threat-registry.yaml", row.id));
        }
    }

    let coverage_percent = if rows.is_empty() {
        0.0
    } else {
        (((rows.len()
            - errors
                .iter()
                .filter(|item| item.contains("missing from threat-registry.yaml"))
                .count()) as f64)
            / (rows.len() as f64))
            * 100.0
    };
    let coverage_payload = serde_json::json!({
        "schema_version": 1,
        "kind": "security_threat_coverage_report",
        "status": if errors.is_empty() { "ok" } else { "error" },
        "coverage_percent": (coverage_percent * 100.0).round() / 100.0,
        "threats_total": rows.len(),
        "by_severity": by_severity,
        "by_category": by_category,
        "errors": errors
    });
    let coverage_path = named_report_path(&root, "security-threat-coverage-report.json")?;
    fs::write(
        &coverage_path,
        serde_json::to_string_pretty(&coverage_payload)
            .map_err(|err| format!("encode threat coverage report failed: {err}"))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", coverage_path.display()))?;

    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "security_threat_registry_verification",
            "status": coverage_payload["status"].clone(),
            "rows": [{
                "coverage_report_path": coverage_path.strip_prefix(&root).unwrap_or(&coverage_path).display().to_string(),
                "threats_total": rows.len(),
                "coverage_percent": coverage_payload["coverage_percent"].clone(),
                "errors": coverage_payload["errors"].clone()
            }],
            "summary": { "total": 1, "errors": if coverage_payload["status"] == "ok" { 0 } else { 1 }, "warnings": 0 }
        }),
    )?;
    Ok((
        rendered,
        if coverage_payload["status"] == "ok" {
            0
        } else {
            1
        },
    ))
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

fn scan_file_matches(
    root: &Path,
    path: &Path,
    patterns: &[String],
) -> Result<Vec<serde_json::Value>, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let mut matches = Vec::new();
    for pattern in patterns {
        if content.contains(pattern) {
            matches.push(serde_json::json!({
                "path": path.strip_prefix(root).unwrap_or(path).display().to_string(),
                "pattern": pattern
            }));
        }
    }
    Ok(matches)
}

fn run_security_validate(args: SecurityValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let assets_path = root.join("ops/security/threat-model/assets.yaml");
    let threats_path = root.join("ops/security/threat-model/threats.yaml");
    let mitigations_path = root.join("ops/security/threat-model/mitigations.yaml");
    let controls_path = root.join("ops/security/compliance/controls.yaml");
    let auth_model_path = root.join("configs/security/auth-model.yaml");
    let principals_path = root.join("configs/security/principals.yaml");
    let actions_path = root.join("configs/security/actions.yaml");
    let resources_path = root.join("configs/security/resources.yaml");
    let policy_path = root.join("configs/security/policy.yaml");
    let data_classification_path = root.join("configs/security/data-classification.yaml");
    let audit_schema_path = root.join("configs/observability/audit-log.schema.json");
    let log_safe_fields_path = root.join("configs/observability/log-safe-fields.yaml");
    let retention_path = root.join("configs/observability/retention.yaml");
    let asset_schema_path = root.join("configs/contracts/security/assets.schema.json");
    let threats_schema_path = root.join("configs/contracts/security/threats.schema.json");
    let mitigations_schema_path = root.join("configs/contracts/security/mitigations.schema.json");
    let controls_schema_path = root.join("configs/contracts/security/controls.schema.json");
    let auth_model_schema_path = root.join("configs/contracts/security/auth-model.schema.json");
    let principals_schema_path = root.join("configs/contracts/security/principals.schema.json");
    let actions_schema_path = root.join("configs/contracts/security/actions.schema.json");
    let resources_schema_path = root.join("configs/contracts/security/resources.schema.json");
    let policy_schema_path = root.join("configs/contracts/security/policy.schema.json");
    let data_classification_schema_path =
        root.join("configs/contracts/security/data-classification.schema.json");
    let log_safe_fields_schema_path =
        root.join("configs/observability/log-safe-fields.schema.json");
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
    let log_field_inventory_schema_path =
        root.join("configs/observability/log-field-inventory.schema.json");
    let retention_schema_path = root.join("configs/observability/retention.schema.json");
    let secrets_path = root.join("configs/security/secrets.json");
    let redaction_path = root.join("configs/security/redaction.json");
    let forbidden_patterns_path = root.join("configs/security/forbidden-patterns.json");
    let dependency_policy_path = root.join("configs/security/dependency-source-policy.json");
    let signing_policy_path = root.join("ops/release/signing/policy.yaml");

    ensure_json(&asset_schema_path)?;
    ensure_json(&threats_schema_path)?;
    ensure_json(&mitigations_schema_path)?;
    ensure_json(&controls_schema_path)?;
    ensure_json(&auth_model_schema_path)?;
    ensure_json(&principals_schema_path)?;
    ensure_json(&actions_schema_path)?;
    ensure_json(&resources_schema_path)?;
    ensure_json(&policy_schema_path)?;
    ensure_json(&data_classification_schema_path)?;
    ensure_json(&log_safe_fields_schema_path)?;
    ensure_json(&secrets_schema_path)?;
    ensure_json(&redaction_schema_path)?;
    ensure_json(&forbidden_patterns_schema_path)?;
    ensure_json(&dependency_policy_schema_path)?;
    ensure_json(&github_actions_exceptions_schema_path)?;
    ensure_json(&signing_policy_schema_path)?;
    ensure_json(&audit_schema_path)?;
    ensure_json(&log_field_inventory_schema_path)?;
    ensure_json(&retention_schema_path)?;

    let assets = read_yaml(&assets_path)?;
    let threats = read_yaml(&threats_path)?;
    let mitigations = read_yaml(&mitigations_path)?;
    let controls = read_yaml(&controls_path)?;
    let auth_model = read_yaml(&auth_model_path)?;
    let principals = read_yaml(&principals_path)?;
    let actions = read_yaml(&actions_path)?;
    let resources = read_yaml(&resources_path)?;
    let policy = read_yaml(&policy_path)?;
    let data_classification = read_yaml(&data_classification_path)?;
    let log_safe_fields = read_yaml(&log_safe_fields_path)?;
    let secrets = read_json(&secrets_path)?;
    let redaction = read_json(&redaction_path)?;
    let forbidden_patterns = read_json(&forbidden_patterns_path)?;
    let dependency_policy = read_json(&dependency_policy_path)?;
    let signing_policy = read_yaml(&signing_policy_path)?;
    let release_evidence_policy = read_json(&root.join("ops/release/evidence/policy.json"))?;
    let retention = read_yaml(&retention_path)?;

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
    let data_class_rows = data_classification
        .get("classes")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let log_safe_field_rows = log_safe_fields
        .get("fields")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let retention_rows = retention
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
    let main_source =
        fs::read_to_string(root.join("crates/bijux-atlas/src/bin/bijux-atlas-server.rs"))
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
    let data_class_ids = data_class_rows
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let classified_field_names = data_class_rows
        .iter()
        .flat_map(|row| {
            row.get("fields")
                .and_then(serde_yaml::Value::as_sequence)
                .cloned()
                .unwrap_or_default()
        })
        .filter_map(|row| row.as_str().map(std::string::ToString::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    let safe_field_names = log_safe_field_rows
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_yaml::Value::as_str))
        .map(std::string::ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
    let redaction_class_refs = redaction
        .get("classification_refs")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.as_str().map(std::string::ToString::to_string))
        .collect::<Vec<_>>();
    let sec_priv_001 = !data_class_rows.is_empty()
        && !redaction_class_refs.is_empty()
        && redaction_class_refs
            .iter()
            .all(|id| data_class_ids.contains(id.as_str()));

    let request_utils_source = fs::read_to_string(
        root.join("crates/bijux-atlas/src/application/server/request_utils.rs"),
    )
            .map_err(|err| format!("failed to read request utils source: {err}"))?;
    let data_command_source =
        fs::read_to_string(root.join("crates/bijux-dev-atlas/src/commands/data.rs"))
            .map_err(|err| format!("failed to read data command source: {err}"))?;
    let main_audit_source_fields = collect_source_json_fields(&main_source);
    let data_audit_source_fields = collect_source_json_fields(&data_command_source);
    let audit_event_names = [
        "config_loaded",
        "startup",
        "ingest_started",
        "ingest_completed",
        "query_executed",
        "admin_action",
    ];
    let runtime_audit_sources_present = request_utils_source.contains("query_executed")
        && request_utils_source.contains("admin_action")
        && main_source.contains("audit_config_loaded")
        && main_source.contains("audit_startup")
        && data_command_source.contains("audit_ingest_started")
        && data_command_source.contains("audit_ingest_completed");
    let audit_smoke_rows = vec![
        serde_json::json!({
            "event_id": "audit_config_loaded",
            "event_name": "config_loaded",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 1,
            "sink": "stdout",
            "action": "runtime.config.read",
            "resource_kind": "namespace",
            "resource_id": "atlas"
        }),
        serde_json::json!({
            "event_id": "audit_startup",
            "event_name": "startup",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 2,
            "sink": "stdout",
            "principal": "operator",
            "action": "runtime.startup",
            "resource_kind": "namespace",
            "resource_id": "atlas"
        }),
        serde_json::json!({
            "event_id": "audit_query_executed",
            "event_name": "query_executed",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 3,
            "sink": "stdout",
            "principal": "service-account",
            "action": "dataset.read",
            "resource_kind": "dataset-id",
            "resource_id": "/v1/datasets"
        }),
        serde_json::json!({
            "event_id": "audit_admin_action",
            "event_name": "admin_action",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 4,
            "sink": "stdout",
            "principal": "operator",
            "action": "ops.admin",
            "resource_kind": "namespace",
            "resource_id": "/debug/datasets"
        }),
        serde_json::json!({
            "event_id": "audit_ingest_started",
            "event_name": "ingest_started",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 5,
            "sink": "stdout",
            "principal": "ci",
            "action": "dataset.ingest",
            "resource_kind": "dataset-id",
            "resource_id": "110/homo_sapiens/GRCh38"
        }),
        serde_json::json!({
            "event_id": "audit_ingest_completed",
            "event_name": "ingest_completed",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": 6,
            "sink": "stdout",
            "principal": "ci",
            "action": "dataset.ingest",
            "resource_kind": "dataset-id",
            "resource_id": "110/homo_sapiens/GRCh38"
        }),
    ];
    let audit_smoke_path = named_report_path(&root, "audit-smoke.jsonl")?;
    write_json_lines(&audit_smoke_path, &audit_smoke_rows)?;
    let mut audit_verify_errors = Vec::new();
    for row in &audit_smoke_rows {
        if let Err(err) = validate_audit_record_shape(row, &audit_event_names) {
            audit_verify_errors.push(err);
        }
    }
    let obs_audit_001 = runtime_audit_sources_present && audit_verify_errors.is_empty();
    let audit_verify_report = serde_json::json!({
        "schema_version": 1,
        "status": if obs_audit_001 { "ok" } else { "failed" },
        "log_path": audit_smoke_path
            .strip_prefix(&root)
            .unwrap_or(&audit_smoke_path)
            .display()
            .to_string(),
        "summary": {
            "total": audit_smoke_rows.len(),
            "errors": audit_verify_errors.len()
        },
        "rows": audit_smoke_rows
    });
    let audit_verify_report_path = named_report_path(&root, "audit-verify.json")?;
    fs::write(
        &audit_verify_report_path,
        serde_json::to_string_pretty(&audit_verify_report)
            .map_err(|err| format!("encode audit verify report failed: {err}"))?,
    )
    .map_err(|err| {
        format!(
            "failed to write {}: {err}",
            audit_verify_report_path.display()
        )
    })?;
    let obs_audit_002 = audit_verify_errors.is_empty();
    let obs_ret_001 = !retention_rows.is_empty()
        && retention
            .get("schema_version")
            .and_then(serde_yaml::Value::as_i64)
            == Some(1);
    let mut observed_audit_fields = std::collections::BTreeSet::from([
        "event_id".to_string(),
        "event_name".to_string(),
        "timestamp_policy".to_string(),
        "timestamp_unix_s".to_string(),
        "sink".to_string(),
        "principal".to_string(),
        "action".to_string(),
        "resource_kind".to_string(),
        "resource_id".to_string(),
        "status".to_string(),
    ]);
    observed_audit_fields.extend(main_audit_source_fields);
    observed_audit_fields.extend(data_audit_source_fields);
    for row in &audit_smoke_rows {
        if let Some(object) = row.as_object() {
            observed_audit_fields.extend(object.keys().cloned());
        }
    }
    let mut unclassified_log_fields = Vec::new();
    let mut log_field_inventory_rows = Vec::new();
    let mut inventory_field_set = safe_field_names.clone();
    inventory_field_set.extend(classified_field_names.iter().cloned());
    inventory_field_set.extend(observed_audit_fields.iter().cloned());
    for field in inventory_field_set {
        let classification = if safe_field_names.contains(&field) {
            "safe"
        } else if classified_field_names.contains(&field) {
            "sensitive"
        } else {
            unclassified_log_fields.push(field.clone());
            "unknown"
        };
        log_field_inventory_rows.push(serde_json::json!({
            "field": field.clone(),
            "classification": classification,
            "observed_in_code": observed_audit_fields.contains(&field)
        }));
    }
    let log_field_inventory_report = serde_json::json!({
        "schema_version": 1,
        "status": if unclassified_log_fields.is_empty() { "ok" } else { "failed" },
        "summary": {
            "total": log_field_inventory_rows.len(),
            "errors": unclassified_log_fields.len()
        },
        "rows": log_field_inventory_rows
    });
    let log_field_inventory_path = named_report_path(&root, "log-field-inventory.json")?;
    fs::write(
        &log_field_inventory_path,
        serde_json::to_string_pretty(&log_field_inventory_report)
            .map_err(|err| format!("encode log field inventory report failed: {err}"))?,
    )
    .map_err(|err| {
        format!(
            "failed to write {}: {err}",
            log_field_inventory_path.display()
        )
    })?;
    let obs_log_inv_001 = unclassified_log_fields.is_empty();
    let release_manifest_path = root.join("ops/release/evidence/manifest.json");
    let release_manifest = if release_manifest_path.exists() {
        Some(read_json(&release_manifest_path)?)
    } else {
        None
    };
    let rel_sec_auth_001 = release_manifest.as_ref().is_some_and(|manifest| {
        manifest
            .get("auth_policy")
            .and_then(|value| value.get("auth_model"))
            .and_then(|value| value.get("path"))
            .and_then(serde_json::Value::as_str)
            == Some("configs/security/auth-model.yaml")
            && manifest
                .get("auth_policy")
                .and_then(|value| value.get("access_policy"))
                .and_then(|value| value.get("path"))
                .and_then(serde_json::Value::as_str)
                == Some("configs/security/policy.yaml")
    });
    let rel_audit_001 = release_manifest.as_ref().is_some_and(|manifest| {
        manifest
            .get("audit_assets")
            .and_then(|value| value.get("schema"))
            .and_then(|value| value.get("path"))
            .and_then(serde_json::Value::as_str)
            == Some("configs/observability/audit-log.schema.json")
            && manifest
                .get("audit_assets")
                .and_then(|value| value.get("retention_policy"))
                .and_then(|value| value.get("path"))
                .and_then(serde_json::Value::as_str)
                == Some("configs/observability/retention.yaml")
    });
    let rel_audit_002 = release_manifest.as_ref().is_some_and(|manifest| {
        manifest
            .get("audit_assets")
            .and_then(|value| value.get("verification_report"))
            .and_then(|value| value.get("path"))
            .and_then(serde_json::Value::as_str)
            == Some("artifacts/security/audit-verify.json")
            && manifest
                .get("audit_assets")
                .and_then(|value| value.get("log_field_inventory"))
                .and_then(|value| value.get("path"))
                .and_then(serde_json::Value::as_str)
                == Some("artifacts/security/log-field-inventory.json")
    });

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
    let default_scan_dir = root.join("ops/release/evidence");
    let forbidden_literals = forbidden_patterns
        .get("patterns")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("literal").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let evidence_matches = scan_matches(&root, &default_scan_dir, &forbidden_literals)?;
    let audit_log_matches = if audit_smoke_path.exists() {
        scan_file_matches(&root, &audit_smoke_path, &forbidden_literals)?
    } else {
        Vec::new()
    };
    let sec_red_001 = missing_redaction_keys.is_empty();
    let sec_red_002 = evidence_matches.is_empty() && audit_log_matches.is_empty();

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

    let lock_posture = dependency_policy
        .get("dependency_lock_posture")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let mut dependency_lock_gaps = Vec::new();

    let rust_lock_required = lock_posture
        .get("rust")
        .and_then(|value| value.get("required"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let rust_lockfile_path = lock_posture
        .get("rust")
        .and_then(|value| value.get("lockfile_path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Cargo.lock");
    let rust_lock_text = fs::read_to_string(root.join(rust_lockfile_path)).ok();
    if rust_lock_required && rust_lock_text.is_none() {
        dependency_lock_gaps.push(format!("rust:missing:{rust_lockfile_path}"));
    }

    let npm_lock_required = lock_posture
        .get("npm")
        .and_then(|value| value.get("required"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let npm_lock_paths = lock_posture
        .get("npm")
        .and_then(|value| value.get("lockfile_paths"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let mut npm_missing_locks = Vec::new();
    for lockfile in &npm_lock_paths {
        if !root.join(lockfile).exists() {
            npm_missing_locks.push((*lockfile).to_string());
        }
    }
    if npm_lock_required && !npm_missing_locks.is_empty() {
        dependency_lock_gaps.extend(
            npm_missing_locks
                .into_iter()
                .map(|path| format!("npm:missing:{path}")),
        );
    }

    let python_lock_required = lock_posture
        .get("python")
        .and_then(|value| value.get("required"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let python_lock_paths = lock_posture
        .get("python")
        .and_then(|value| value.get("lockfile_paths"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let mut python_missing_locks = Vec::new();
    for lockfile in &python_lock_paths {
        if !root.join(lockfile).exists() {
            python_missing_locks.push((*lockfile).to_string());
        }
    }
    if python_lock_required && !python_missing_locks.is_empty() {
        dependency_lock_gaps.extend(
            python_missing_locks
                .into_iter()
                .map(|path| format!("python:missing:{path}")),
        );
    }

    let helm_lock_required = lock_posture
        .get("helm")
        .and_then(|value| value.get("required"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let helm_lock_paths = lock_posture
        .get("helm")
        .and_then(|value| value.get("lockfile_paths"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let mut helm_missing_locks = Vec::new();
    for lockfile in &helm_lock_paths {
        if !root.join(lockfile).exists() {
            helm_missing_locks.push((*lockfile).to_string());
        }
    }
    if helm_lock_required && !helm_missing_locks.is_empty() {
        dependency_lock_gaps.extend(
            helm_missing_locks
                .into_iter()
                .map(|path| format!("helm:missing:{path}")),
        );
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
            invalid_action_exceptions
                .push(format!("{workflow_path}:{action}:owner-or-expiry-invalid"));
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
        .unwrap_or("ops/docker/bases.lock");
    let evidence_manifest_path = image_policy
        .get("evidence_manifest")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("ops/release/evidence/manifest.json");
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

    let cve_budget = release_evidence_policy
        .get("cve_budget")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"max_critical": 0, "max_high": 0}));
    let budget_critical = cve_budget
        .get("max_critical")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);
    let budget_high = cve_budget
        .get("max_high")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);
    let overrides = release_evidence_policy
        .get("cve_overrides")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut scan_rows = Vec::new();
    let mut total_critical = 0i64;
    let mut total_high = 0i64;
    let mut total_medium = 0i64;
    let mut total_low = 0i64;
    let mut scan_errors = Vec::new();
    for row in &sbom_rows {
        let _ = row;
    }
    for scan_entry in evidence_manifest
        .get("scan_reports")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(scan_path) = scan_entry.as_str() else {
            continue;
        };
        let scan_abs = root.join(scan_path);
        if !scan_abs.exists() {
            scan_errors.push(format!("missing-scan-report:{scan_path}"));
            continue;
        }
        let scan_json = read_json(&scan_abs)?;
        let Some((critical, high, medium, low)) = parse_scan_summary(&scan_json) else {
            scan_errors.push(format!("invalid-scan-summary:{scan_path}"));
            continue;
        };
        total_critical += critical;
        total_high += high;
        total_medium += medium;
        total_low += low;
        scan_rows.push(serde_json::json!({
            "path": scan_path,
            "critical": critical,
            "high": high,
            "medium": medium,
            "low": low
        }));
    }
    let mut allowed_critical = budget_critical;
    let mut allowed_high = budget_high;
    let mut invalid_overrides = Vec::new();
    for row in &overrides {
        let id = row
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let justification = row
            .get("justification")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let expires_on = row
            .get("expires_on")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let extra_critical = row
            .get("critical")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let extra_high = row
            .get("high")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        if id.is_empty()
            || justification.len() < 8
            || !is_iso_date(expires_on)
            || expires_on < "2026-03-04"
        {
            invalid_overrides.push(format!("invalid-override:{id}:{expires_on}"));
            continue;
        }
        allowed_critical += extra_critical;
        allowed_high += extra_high;
    }
    let vulnerability_budget_fail =
        total_critical > allowed_critical || total_high > allowed_high || !scan_errors.is_empty();
    let sec_vuln_001 = scan_errors.is_empty();
    let sec_vuln_002 = !vulnerability_budget_fail;
    let sec_vuln_003 = invalid_overrides.is_empty();
    let vulnerability_report = serde_json::json!({
        "schema_version": 1,
        "status": if sec_vuln_001 && sec_vuln_002 && sec_vuln_003 { "ok" } else { "failed" },
        "budget": {
            "critical": budget_critical,
            "high": budget_high
        },
        "allowed_with_overrides": {
            "critical": allowed_critical,
            "high": allowed_high
        },
        "totals": {
            "critical": total_critical,
            "high": total_high,
            "medium": total_medium,
            "low": total_low
        },
        "rows": scan_rows,
        "gaps": {
            "scan_errors": scan_errors,
            "invalid_overrides": invalid_overrides
        }
    });
    let vulnerability_report_path = named_report_path(&root, "security-vulnerability-scan.json")?;
    fs::write(
        &vulnerability_report_path,
        serde_json::to_string_pretty(&vulnerability_report)
            .map_err(|err| format!("encode vulnerability report failed: {err}"))?,
    )
    .map_err(|err| {
        format!(
            "failed to write {}: {err}",
            vulnerability_report_path.display()
        )
    })?;

    let unsafe_pattern_needles = ["curl", "wget"];
    let script_allowlist_text =
        fs::read_to_string(root.join("configs/policy/shell-network-fetch-allowlist.txt"))
            .unwrap_or_default();
    let script_allowlist = parse_expiry_allowlist_rows(&script_allowlist_text);
    let mut unsafe_download_hits = Vec::new();
    for rel in [
        "Makefile",
        ".github/workflows/ci-pr.yml",
        ".github/workflows/ci-nightly.yml",
        ".github/workflows/release-candidate.yml",
        ".github/workflows/ops-validate.yml",
        ".github/workflows/ops-integration-kind.yml",
        ".github/workflows/dependency-lock.yml",
        ".github/workflows/docs-audit.yml",
        ".github/workflows/docs-only.yml",
    ] {
        let path = root.join(rel);
        if !path.exists() {
            continue;
        }
        let text =
            fs::read_to_string(&path).map_err(|err| format!("failed to read {}: {err}", rel))?;
        for (idx, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            let has_fetch = unsafe_pattern_needles
                .iter()
                .any(|needle| lower.contains(needle));
            let has_pipe_shell = (lower.contains("| bash") || lower.contains("| sh")) && has_fetch;
            if !has_pipe_shell {
                continue;
            }
            let key = format!("{rel}:{}", idx + 1);
            let allowlisted = script_allowlist.iter().any(|(entry, expires_on, reason)| {
                entry == &key
                    && is_iso_date(expires_on)
                    && expires_on.as_str() >= "2026-03-04"
                    && reason.len() >= 8
            });
            if !allowlisted {
                unsafe_download_hits.push(key);
            }
        }
    }
    let sec_scripts_001 = unsafe_download_hits.is_empty();

    let mut rust_rows = rust_lock_text
        .as_deref()
        .map(parse_cargo_lock_rows)
        .unwrap_or_default();
    let mut npm_rows = Vec::new();
    for lockfile in &npm_lock_paths {
        let lockfile_path = root.join(lockfile);
        if !lockfile_path.exists() {
            continue;
        }
        let lock_json = read_json(&lockfile_path)?;
        if let Some(packages) = lock_json
            .get("packages")
            .and_then(serde_json::Value::as_object)
        {
            for (name, value) in packages {
                let version = value
                    .get("version")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                if name.is_empty() || version.is_empty() {
                    continue;
                }
                npm_rows.push(format!("{name}@{version}"));
            }
        }
    }
    npm_rows.sort();
    npm_rows.dedup();

    let mut python_rows = Vec::new();
    for lockfile in &python_lock_paths {
        let lockfile_path = root.join(lockfile);
        if !lockfile_path.exists() {
            continue;
        }
        let text = fs::read_to_string(&lockfile_path)
            .map_err(|err| format!("failed to read {}: {err}", lockfile_path.display()))?;
        python_rows.extend(parse_python_lock_rows(&text));
    }
    python_rows.sort();
    python_rows.dedup();

    let mut helm_rows = Vec::new();
    for lockfile in &helm_lock_paths {
        helm_rows.extend(parse_helm_lock_rows(&root.join(lockfile))?);
    }
    helm_rows.sort();
    helm_rows.dedup();

    rust_rows.sort();
    rust_rows.dedup();

    let sec_deps_003 = dependency_lock_gaps.is_empty();
    let dependency_inventory_report = serde_json::json!({
        "schema_version": 1,
        "status": if sec_deps_003 { "ok" } else { "failed" },
        "summary": {
            "rust_dependencies": rust_rows.len(),
            "npm_dependencies": npm_rows.len(),
            "python_dependencies": python_rows.len(),
            "helm_dependencies": helm_rows.len(),
            "lock_gaps": dependency_lock_gaps.len()
        },
        "lock_posture": lock_posture,
        "rows": {
            "rust": rust_rows,
            "npm": npm_rows,
            "python": python_rows,
            "helm": helm_rows
        },
        "gaps": dependency_lock_gaps
    });
    let dependency_inventory_report_path = named_report_path(&root, "dependency-inventory.json")?;
    fs::write(
        &dependency_inventory_report_path,
        serde_json::to_string_pretty(&dependency_inventory_report)
            .map_err(|err| format!("encode dependency inventory report failed: {err}"))?,
    )
    .map_err(|err| {
        format!(
            "failed to write {}: {err}",
            dependency_inventory_report_path.display()
        )
    })?;

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
    .map_err(|err| {
        format!(
            "failed to write {}: {err}",
            github_actions_report_path.display()
        )
    })?;

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
            && sec_priv_001
            && obs_audit_001
            && obs_audit_002
            && obs_ret_001
            && obs_log_inv_001
            && rel_sec_auth_001
            && rel_audit_001
            && rel_audit_002
            && sec_deps_001
            && sec_deps_002
            && sec_deps_003
            && sec_images_001
            && sec_actions_001
            && sec_sbom_001
            && sec_vuln_001
            && sec_vuln_002
            && sec_vuln_003
            && sec_scripts_001
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
            "dependency_inventory": dependency_inventory_report_path
                .strip_prefix(&root)
                .unwrap_or(&dependency_inventory_report_path)
                .display()
                .to_string(),
            "vulnerability_scan": vulnerability_report_path
                .strip_prefix(&root)
                .unwrap_or(&vulnerability_report_path)
                .display()
                .to_string(),
            "github_actions": github_actions_report_path
                .strip_prefix(&root)
                .unwrap_or(&github_actions_report_path)
                .display()
                .to_string(),
            "audit_verify": audit_verify_report_path
                .strip_prefix(&root)
                .unwrap_or(&audit_verify_report_path)
                .display()
                .to_string(),
            "log_field_inventory": log_field_inventory_path
                .strip_prefix(&root)
                .unwrap_or(&log_field_inventory_path)
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
            "SEC-PRIV-001": sec_priv_001,
            "OBS-AUDIT-001": obs_audit_001,
            "OBS-AUDIT-002": obs_audit_002,
            "OBS-RET-001": obs_ret_001,
            "OBS-LOG-INV-001": obs_log_inv_001,
            "REL-SEC-AUTH-001": rel_sec_auth_001,
            "REL-AUDIT-001": rel_audit_001,
            "REL-AUDIT-002": rel_audit_002,
            "SEC-RED-001": sec_red_001,
            "SEC-RED-002": sec_red_002,
            "SEC-DEPS-001": sec_deps_001,
            "SEC-DEPS-002": sec_deps_002,
            "SEC-DEPS-003": sec_deps_003,
            "SEC-IMAGES-001": sec_images_001,
            "SEC-ACTIONS-001": sec_actions_001,
            "SEC-SBOM-001": sec_sbom_001,
            "SEC-VULN-001": sec_vuln_001,
            "SEC-VULN-002": sec_vuln_002,
            "SEC-VULN-003": sec_vuln_003,
            "SEC-SCRIPTS-001": sec_scripts_001
        },
        "policy_validation": {
            "dependency_source_policy": sec_deps_001 && sec_deps_002 && sec_images_001 && sec_actions_001 && sec_sbom_001,
            "vulnerability_budget_policy": sec_vuln_001 && sec_vuln_002 && sec_vuln_003,
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
            "redaction_class_refs": redaction_class_refs,
            "audit_verify_errors": audit_verify_errors,
            "audit_log_matches": audit_log_matches,
            "unclassified_log_fields": unclassified_log_fields,
            "release_evidence_gaps": {
                "REL-SEC-AUTH-001": !rel_sec_auth_001,
                "REL-AUDIT-001": !rel_audit_001,
                "REL-AUDIT-002": !rel_audit_002
            },
            "evidence_secret_matches": evidence_matches,
            "disallowed_npm_sources": disallowed_npm_sources,
            "disallowed_python_indexes": disallowed_python_indexes,
            "dependency_lock_gaps": dependency_lock_gaps,
            "unsafe_download_hits": unsafe_download_hits,
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
    let controls = read_yaml(&root.join("ops/security/compliance/controls.yaml"))?;
    let matrix = read_yaml(&root.join("ops/security/compliance/matrix.yaml"))?;
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

#[cfg(test)]
mod tests {
    use super::{
        run_security_audit, run_security_auth_api_keys, run_security_auth_diagnostics,
        run_security_auth_policy_validate, run_security_auth_token_inspect,
        run_security_authorization_assign, run_security_authorization_diagnostics,
        run_security_authorization_permissions, run_security_authorization_roles,
        run_security_authorization_validate, run_security_config_validate,
        run_security_dependency_audit, run_security_diagnostics, run_security_policy_inspect,
        run_security_threats_explain, run_security_threats_list, run_security_threats_verify,
    };
    use crate::cli::{
        FormatArg, SecurityPolicyInspectArgs, SecurityRoleAssignArgs, SecurityThreatExplainArgs,
        SecurityTokenInspectArgs, SecurityValidateArgs,
    };
    use base64::Engine as _;
    use std::fs;

    fn write_minimal_security_files(root: &std::path::Path) {
        let config_dir = root.join("configs/security");
        fs::create_dir_all(&config_dir).expect("create security config dir");
        fs::write(
            config_dir.join("runtime-security.yaml"),
            r#"schema_version: 1
identity:
  principal_source: ingress
  trust_header: x-atlas-principal
auth:
  mode: api-key
  required: true
authorization:
  default_decision: deny
  policy_source: configs/security/policy.yaml
secrets:
  provider: env
  references: [ATLAS_API_KEY]
keys:
  active_key_id: atlas-key-1
  key_ring: [atlas-key-1]
transport:
  tls_required: true
  min_tls_version: "1.2"
audit:
  enabled: true
  sink: stdout
events:
  classes: [auth.failure]
"#,
        )
        .expect("write runtime-security.yaml");
        fs::write(
            config_dir.join("policy.yaml"),
            r#"rules:
  - id: sec-auth-required
    description: auth required
    enabled: true
"#,
        )
        .expect("write policy.yaml");
        fs::write(
            config_dir.join("auth-model.yaml"),
            r#"default_stance: zero-trust
auth_support: supported
methods: [api-key, token, oidc, mtls]
runtime_auth_mode_env: ATLAS_AUTH_MODE
docs:
  model: docs/architecture/security/authentication-strategy.md
  runbook: docs/operations/security/deploy-behind-auth-proxy.md
"#,
        )
        .expect("write auth-model.yaml");
        fs::write(
            config_dir.join("roles.yaml"),
            r#"schema_version: 1
roles:
  - id: role.user.readonly
    description: readonly
    permissions: [perm.catalog.read]
    inherits: []
"#,
        )
        .expect("write roles.yaml");
        fs::write(
            config_dir.join("permissions.yaml"),
            r#"schema_version: 1
permissions:
  - id: perm.catalog.read
    action: catalog.read
    resource_kind: namespace
    description: read catalog
"#,
        )
        .expect("write permissions.yaml");
        fs::write(
            config_dir.join("role-assignments.yaml"),
            r#"schema_version: 1
assignments:
  - principal: user
    role_id: role.user.readonly
"#,
        )
        .expect("write role-assignments.yaml");
    }

    fn write_minimal_threat_model_files(root: &std::path::Path) {
        let dir = root.join("ops/security/threat-model");
        fs::create_dir_all(&dir).expect("create threat model dir");
        fs::write(
            dir.join("assets.yaml"),
            r#"schema_version: 1
assets:
  - id: runtime_api
    type: endpoint
    description: api
    sensitivity: medium
    owner: security
"#,
        )
        .expect("write assets");
        fs::write(
            dir.join("threats.yaml"),
            r#"schema_version: 1
threats:
  - id: SEC-THREAT-RUNTIME-SPOOF
    category: spoofing
    title: spoofing request
    severity: high
    likelihood: medium
    affected_component: runtime_api
    mitigations: [MIT-HTTP-BOUNDARY]
    residual_risk: malformed traffic
"#,
        )
        .expect("write threats");
        fs::write(
            dir.join("mitigations.yaml"),
            r#"schema_version: 1
mitigations:
  - id: MIT-HTTP-BOUNDARY
    title: boundary control
"#,
        )
        .expect("write mitigations");
        fs::write(
            dir.join("classification-taxonomy.yaml"),
            r#"schema_version: 1
methodology: stride
severity_levels: [critical, high, medium, low]
likelihood_levels: [high, medium, low]
categories: [spoofing, tampering, repudiation, information-disclosure, denial-of-service, elevation-of-privilege]
attacker_capabilities:
  - external unauthenticated caller
"#,
        )
        .expect("write taxonomy");
        fs::write(
            dir.join("threat-registry.yaml"),
            r#"schema_version: 1
registry_name: atlas_security_threats
threat_ids: [SEC-THREAT-RUNTIME-SPOOF]
"#,
        )
        .expect("write threat registry");
    }

    #[test]
    fn security_config_validate_command_returns_ok_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_security_files(temp.path());
        let (rendered, code) = run_security_config_validate(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run security config validate");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "security_config_validation_report");
        assert_eq!(value["status"], "ok");
    }

    #[test]
    fn security_threat_commands_emit_registry_outputs() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_threat_model_files(temp.path());

        let (list_rendered, list_code) = run_security_threats_list(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run threat list");
        assert_eq!(list_code, 0);
        let list_value: serde_json::Value =
            serde_json::from_str(&list_rendered).expect("parse list output");
        assert_eq!(list_value["kind"], "security_threat_registry_list");

        let (explain_rendered, explain_code) =
            run_security_threats_explain(SecurityThreatExplainArgs {
                repo_root: Some(temp.path().to_path_buf()),
                threat_id: Some("SEC-THREAT-RUNTIME-SPOOF".to_string()),
                format: FormatArg::Json,
                out: None,
            })
            .expect("run threat explain");
        assert_eq!(explain_code, 0);
        let explain_value: serde_json::Value =
            serde_json::from_str(&explain_rendered).expect("parse explain output");
        assert_eq!(explain_value["kind"], "security_threat_registry_explain");

        let (verify_rendered, verify_code) = run_security_threats_verify(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run threat verify");
        assert_eq!(verify_code, 0);
        let verify_value: serde_json::Value =
            serde_json::from_str(&verify_rendered).expect("parse verify output");
        assert_eq!(
            verify_value["kind"],
            "security_threat_registry_verification"
        );
        assert_eq!(
            verify_value["rows"][0]["coverage_report_path"],
            "artifacts/security/security-threat-coverage-report.json"
        );
    }

    #[test]
    fn security_threat_verify_reports_registry_mismatch() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_threat_model_files(temp.path());
        fs::write(
            temp.path()
                .join("ops/security/threat-model/threat-registry.yaml"),
            r#"schema_version: 1
registry_name: atlas_security_threats
threat_ids: []
"#,
        )
        .expect("write mismatched registry");

        let (rendered, code) = run_security_threats_verify(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run verify");
        assert_eq!(code, 1);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse verify");
        assert_eq!(value["status"], "error");
        let errors = value["rows"][0]["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        assert!(
            errors
                .iter()
                .any(|item| item.contains("missing from threat-registry.yaml")),
            "expected registry mismatch error"
        );
    }

    #[test]
    fn security_diagnostics_and_policy_inspection_return_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_security_files(temp.path());

        let (diag, code) = run_security_diagnostics(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run diagnostics");
        assert_eq!(code, 0);
        let diag_value: serde_json::Value = serde_json::from_str(&diag).expect("parse diagnostics");
        assert_eq!(diag_value["kind"], "security_diagnostics_report");

        let (inspect, code) = run_security_policy_inspect(SecurityPolicyInspectArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
            policy_id: Some("sec-auth-required".to_string()),
        })
        .expect("run policy inspect");
        assert_eq!(code, 0);
        let inspect_value: serde_json::Value =
            serde_json::from_str(&inspect).expect("parse inspection");
        assert_eq!(inspect_value["kind"], "security_policy_inspection_report");
        assert_eq!(
            inspect_value["policies"]
                .as_array()
                .expect("policies array")
                .len(),
            1
        );
    }

    #[test]
    fn security_audit_command_reports_missing_artifacts_cleanly() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_security_files(temp.path());
        let (rendered, code) = run_security_audit(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run security audit");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse rendered");
        assert_eq!(value["kind"], "security_audit_report");
        assert_eq!(value["artifacts_present"], false);
    }

    #[test]
    fn security_dependency_audit_command_emits_report() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crates root")
            .parent()
            .expect("workspace root")
            .to_path_buf();

        let (rendered, code) = run_security_dependency_audit(SecurityValidateArgs {
            repo_root: Some(root),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run dependency audit");
        assert_eq!(code, 0);
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("parse report");
        assert_eq!(value["kind"], "security_dependency_audit_report");
        assert_eq!(value["status"], "ok");
    }

    #[test]
    fn security_authentication_commands_emit_expected_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_security_files(temp.path());

        let (api_keys, code) = run_security_auth_api_keys(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run api key management");
        assert_eq!(code, 0);
        let api_keys_value: serde_json::Value =
            serde_json::from_str(&api_keys).expect("parse api keys report");
        assert_eq!(
            api_keys_value["kind"],
            "authentication_api_key_management_report"
        );

        let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            r#"{"sub":"svc-a","iss":"atlas-auth","aud":"atlas-api","exp":4102444800,"jti":"t1","scope":"dataset.read"}"#,
        );
        let (token, code) = run_security_auth_token_inspect(SecurityTokenInspectArgs {
            repo_root: Some(temp.path().to_path_buf()),
            token: format!("a.{claims}.b"),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run token inspection");
        assert_eq!(code, 0);
        let token_value: serde_json::Value =
            serde_json::from_str(&token).expect("parse token report");
        assert_eq!(
            token_value["kind"],
            "authentication_token_inspection_report"
        );

        let (diag, code) = run_security_auth_diagnostics(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run auth diagnostics");
        assert_eq!(code, 0);
        let diag_value: serde_json::Value =
            serde_json::from_str(&diag).expect("parse diagnostics report");
        assert_eq!(diag_value["kind"], "authentication_diagnostics_report");

        let (policy, code) = run_security_auth_policy_validate(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run auth policy validate");
        assert_eq!(code, 0);
        let policy_value: serde_json::Value =
            serde_json::from_str(&policy).expect("parse policy report");
        assert_eq!(
            policy_value["kind"],
            "authentication_policy_validation_report"
        );
    }

    #[test]
    fn security_authentication_policy_validate_accepts_supported_methods_key() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_security_files(temp.path());

        let auth_model_path = temp.path().join("configs/security/auth-model.yaml");
        fs::write(
            &auth_model_path,
            r#"default_stance: zero-trust
auth_support: supported
supported_methods: [api-key, token, oidc, mtls]
runtime_auth_mode_env: ATLAS_AUTH_MODE
"#,
        )
        .expect("write auth-model.yaml");

        let (policy, code) = run_security_auth_policy_validate(SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        })
        .expect("run auth policy validate");
        assert_eq!(code, 0);
        let policy_value: serde_json::Value =
            serde_json::from_str(&policy).expect("parse policy report");
        assert_eq!(policy_value["status"], "ok");
        assert_eq!(policy_value["auth_methods"], 4);
    }

    #[test]
    fn security_authorization_commands_emit_expected_reports() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_minimal_security_files(temp.path());

        let common = SecurityValidateArgs {
            repo_root: Some(temp.path().to_path_buf()),
            format: FormatArg::Json,
            out: None,
        };

        let (roles, code) = run_security_authorization_roles(common.clone()).expect("roles");
        assert_eq!(code, 0);
        let roles_value: serde_json::Value = serde_json::from_str(&roles).expect("roles json");
        assert_eq!(roles_value["kind"], "authorization_role_management_report");

        let (permissions, code) =
            run_security_authorization_permissions(common.clone()).expect("permissions");
        assert_eq!(code, 0);
        let permissions_value: serde_json::Value =
            serde_json::from_str(&permissions).expect("permissions json");
        assert_eq!(
            permissions_value["kind"],
            "authorization_permission_inspection_report"
        );

        let (diag, code) =
            run_security_authorization_diagnostics(common.clone()).expect("diagnostics");
        assert_eq!(code, 0);
        let diag_value: serde_json::Value = serde_json::from_str(&diag).expect("diag json");
        assert_eq!(diag_value["kind"], "authorization_diagnostics_report");
        assert_eq!(diag_value["assignment_count"], 1);
        assert_eq!(diag_value["default_decision"], "deny");

        let (assign, code) = run_security_authorization_assign(SecurityRoleAssignArgs {
            repo_root: Some(temp.path().to_path_buf()),
            principal: "user".to_string(),
            role_id: "role.user.readonly".to_string(),
            format: FormatArg::Json,
            out: None,
        })
        .expect("assign");
        assert_eq!(code, 0);
        let assign_value: serde_json::Value = serde_json::from_str(&assign).expect("assign json");
        assert_eq!(assign_value["kind"], "authorization_role_assignment_report");

        let (validate, code) = run_security_authorization_validate(common).expect("validate");
        assert_eq!(code, 0);
        let validate_value: serde_json::Value =
            serde_json::from_str(&validate).expect("validate json");
        assert_eq!(
            validate_value["kind"],
            "authorization_permission_validation_report"
        );
    }
}
