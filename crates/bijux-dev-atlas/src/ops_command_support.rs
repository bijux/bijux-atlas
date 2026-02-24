use crate::ops_support::StackManifestToml;
use crate::*;
use bijux_dev_atlas_model::OpsRunReport;
use serde::{Deserialize, Serialize};

pub(crate) fn normalize_tool_version_with_regex(raw: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(raw)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ToolMismatchCode {
    MissingBinary,
    VersionMismatch,
}

impl ToolMismatchCode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::MissingBinary => "TOOLS_MISSING_BINARY",
            Self::VersionMismatch => "TOOLS_VERSION_MISMATCH",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ToolsToml {
    pub(crate) tools: Vec<ToolTomlEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ToolTomlEntry {
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) version_regex: String,
    pub(crate) probe_argv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct StackPinsToml {
    pub(crate) charts: std::collections::BTreeMap<String, String>,
    pub(crate) images: std::collections::BTreeMap<String, String>,
    pub(crate) crds: std::collections::BTreeMap<String, String>,
}

pub(crate) fn resolve_ops_root(
    repo_root: &Path,
    ops_root: Option<PathBuf>,
) -> Result<PathBuf, OpsCommandError> {
    let path = ops_root.unwrap_or_else(|| repo_root.join("ops"));
    path.canonicalize().map_err(|err| {
        OpsCommandError::Manifest(format!("cannot resolve ops root {}: {err}", path.display()))
    })
}

pub(crate) fn load_profiles(ops_root: &Path) -> Result<Vec<StackProfile>, OpsCommandError> {
    let path = ops_root.join("stack/profiles.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    let payload: StackProfiles = serde_json::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })?;
    Ok(payload.profiles)
}

fn load_toolchain_inventory(ops_root: &Path) -> Result<ToolchainInventory, OpsCommandError> {
    let path = ops_root.join("inventory/toolchain.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    serde_json::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn load_tools_manifest(repo_root: &Path) -> Result<ToolsToml, OpsCommandError> {
    let path = repo_root.join("ops/tools/tools.toml");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    toml::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn load_stack_pins(repo_root: &Path) -> Result<StackPinsToml, OpsCommandError> {
    let path = repo_root.join("ops/stack/pins.toml");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    toml::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn load_stack_manifest(repo_root: &Path) -> Result<StackManifestToml, OpsCommandError> {
    let path = repo_root.join("ops/stack/stack.toml");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        OpsCommandError::Manifest(format!("failed to read {}: {err}", path.display()))
    })?;
    toml::from_str(&text).map_err(|err| {
        OpsCommandError::Schema(format!("failed to parse {}: {err}", path.display()))
    })
}

pub(crate) fn validate_stack_manifest(
    repo_root: &Path,
    manifest: &StackManifestToml,
) -> Result<Vec<String>, OpsCommandError> {
    let mut errors = Vec::new();
    if manifest.profiles.is_empty() {
        errors.push("stack manifest must declare at least one profile".to_string());
    }
    for (name, profile) in &manifest.profiles {
        let cluster_path = repo_root.join(&profile.cluster_config);
        if !cluster_path.exists() {
            errors.push(format!(
                "stack profile `{name}` references missing cluster config `{}`",
                profile.cluster_config
            ));
        }
        if profile.components.is_empty() {
            errors.push(format!("stack profile `{name}` must declare components"));
            continue;
        }
        let mut sorted = profile.components.clone();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != profile.components.len() {
            errors.push(format!(
                "stack profile `{name}` has duplicate components; ordering must be deterministic"
            ));
        }
        if profile.components != sorted {
            errors.push(format!(
                "stack profile `{name}` components must be lexicographically sorted"
            ));
        }
        for component in &profile.components {
            let component_path = repo_root.join(component);
            if !component_path.exists() {
                errors.push(format!(
                    "stack profile `{name}` references missing component `{component}`"
                ));
            }
        }
    }
    errors.sort();
    errors.dedup();
    Ok(errors)
}

pub(crate) fn parse_tool_overrides(
    values: &[String],
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut out = std::collections::BTreeMap::new();
    for raw in values {
        let Some((name, path)) = raw.split_once('=') else {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        };
        let name = name.trim();
        let path = path.trim();
        if name.is_empty() || path.is_empty() {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        }
        out.insert(name.to_string(), path.to_string());
    }
    Ok(out)
}

pub(crate) fn validate_pins_completeness(
    repo_root: &Path,
    pins: &StackPinsToml,
) -> Result<Vec<String>, OpsCommandError> {
    let mut errors = Vec::new();
    let stack_manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/stack/version-manifest.json")).map_err(
            |err| {
                OpsCommandError::Manifest(format!(
                    "failed to read ops/stack/version-manifest.json: {err}"
                ))
            },
        )?,
    )
    .map_err(|err| OpsCommandError::Schema(format!("invalid version manifest json: {err}")))?;
    if let Some(obj) = stack_manifest.as_object() {
        for (k, v) in obj {
            if k == "schema_version" {
                continue;
            }
            if pins.images.get(k).is_none() {
                errors.push(format!("pins missing image key `{k}`"));
            }
            if let Some(value) = v.as_str() {
                if value.contains(":latest") {
                    errors.push(format!("floating tag forbidden in stack manifest `{k}`"));
                }
            }
        }
    }
    for (k, v) in &pins.images {
        if v.contains(":latest") {
            errors.push(format!("floating tag forbidden in pins image `{k}`"));
        }
    }
    for (k, v) in &pins.charts {
        if v.contains(":latest") {
            errors.push(format!("floating tag forbidden in pins chart `{k}`"));
        }
    }
    let contracts_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/contracts.json")).map_err(
            |err| {
                OpsCommandError::Manifest(format!(
                    "failed to read ops/inventory/contracts.json: {err}"
                ))
            },
        )?,
    )
    .map_err(|err| OpsCommandError::Schema(format!("invalid contracts.json: {err}")))?;
    let contract_paths = contracts_json
        .get("contracts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            v.get("path")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<std::collections::BTreeSet<_>>();
    for required in ["ops/tools/tools.toml", "ops/stack/pins.toml"] {
        if !contract_paths.contains(required) {
            errors.push(format!(
                "contracts inventory missing required entry `{required}`"
            ));
        }
    }
    let values_files = [
        "ops/k8s/charts/bijux-atlas/values.yaml",
        "ops/k8s/charts/bijux-atlas/values-offline.yaml",
    ];
    for file in values_files {
        let text = std::fs::read_to_string(repo_root.join(file))
            .map_err(|err| OpsCommandError::Manifest(format!("failed to read {file}: {err}")))?;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains(":latest") {
                errors.push(format!("floating latest forbidden in {file}: `{trimmed}`"));
            }
            if (trimmed.contains("image:") || trimmed.contains("repository:"))
                && trimmed.contains(':')
                && !trimmed.contains("@sha256:")
                && !trimmed.ends_with(':')
            {
                errors.push(format!(
                    "base image pin must include digest in {file}: `{trimmed}`"
                ));
            }
        }
    }
    let hardcoded_tool_patterns = ["helm ", "kubectl ", "kind ", "k6 "];
    for root in ["makefiles", ".github/workflows"] {
        let walk_root = repo_root.join(root);
        if !walk_root.exists() {
            continue;
        }
        for path in walk_files(&walk_root) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for pattern in hardcoded_tool_patterns {
                if text.contains(pattern) && !text.contains("bijux dev atlas") {
                    let rel = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path.as_path())
                        .display()
                        .to_string();
                    errors.push(format!(
                        "hardcoded tool invocation forbidden (`{pattern}`) in {rel}"
                    ));
                }
            }
        }
    }
    errors.sort();
    errors.dedup();
    Ok(errors)
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if root.is_file() {
        out.push(root.to_path_buf());
        return out;
    }
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            out.extend(walk_files(&entry.path()));
        }
    }
    out
}

pub(crate) fn tool_definitions_sorted(
    inventory: &ToolchainInventory,
) -> Vec<(String, ToolDefinition)> {
    inventory
        .tools
        .iter()
        .map(|(name, definition)| (name.clone(), definition.clone()))
        .collect()
}

pub(crate) fn resolve_profile(
    requested: Option<String>,
    profiles: &[StackProfile],
) -> Result<StackProfile, OpsCommandError> {
    if profiles.is_empty() {
        return Err(OpsCommandError::Profile(
            "no profiles declared in ops/stack/profiles.json".to_string(),
        ));
    }
    if let Some(name) = requested {
        return profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .ok_or_else(|| OpsCommandError::Profile(format!("unknown profile `{name}`")));
    }
    profiles
        .iter()
        .find(|p| p.name == "developer")
        .cloned()
        .or_else(|| profiles.first().cloned())
        .ok_or_else(|| OpsCommandError::Profile("no default profile available".to_string()))
}

pub(crate) fn run_id_or_default(raw: Option<String>) -> Result<RunId, String> {
    raw.map(|v| RunId::parse(&v))
        .transpose()?
        .map_or_else(|| Ok(RunId::from_seed("ops_run")), Ok)
}

pub(crate) fn emit_payload(
    format: FormatArg,
    out: Option<PathBuf>,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let rendered = match format {
        FormatArg::Text => payload
            .get("text")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .unwrap_or_else(|| serde_json::to_string_pretty(payload).unwrap_or_default()),
        FormatArg::Json => serde_json::to_string_pretty(payload).map_err(|err| err.to_string())?,
        FormatArg::Jsonl => {
            if let Some(rows) = payload.get("rows").and_then(|v| v.as_array()) {
                rows.iter()
                    .map(serde_json::to_string)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|err| err.to_string())?
                    .join("\n")
            } else {
                serde_json::to_string(payload).map_err(|err| err.to_string())?
            }
        }
    };
    write_output_if_requested(out, &rendered)?;
    Ok(rendered)
}

pub(crate) mod ops_exit {
    pub const PASS: i32 = 0;
    pub const FAIL: i32 = 1;
    pub const USAGE: i32 = 2;
    pub const INFRA: i32 = 3;
    pub const TOOL_MISSING: i32 = 4;
}

pub(crate) fn build_ops_run_report(
    command: &str,
    common: &OpsCommonArgs,
    run_id: &RunId,
    repo_root: &Path,
    ops_root: &Path,
    suite: Option<String>,
    status: &str,
    exit_code: i32,
    warnings: Vec<String>,
    errors: Vec<String>,
    rows: Vec<serde_json::Value>,
) -> OpsRunReport {
    let mut capabilities = std::collections::BTreeMap::new();
    capabilities.insert(
        "subprocess".to_string(),
        if common.allow_subprocess {
            "enabled: requested by flag".to_string()
        } else {
            "disabled: default deny".to_string()
        },
    );
    capabilities.insert(
        "fs_write".to_string(),
        if common.allow_write {
            "enabled: requested by flag".to_string()
        } else {
            "disabled: default deny".to_string()
        },
    );
    capabilities.insert(
        "network".to_string(),
        if common.allow_network {
            "enabled: requested by flag".to_string()
        } else {
            "disabled: default deny".to_string()
        },
    );
    let mut summary = std::collections::BTreeMap::new();
    summary.insert("warnings".to_string(), warnings.len() as u64);
    summary.insert("errors".to_string(), errors.len() as u64);
    summary.insert("rows".to_string(), rows.len() as u64);
    OpsRunReport {
        schema_version: bijux_dev_atlas_model::schema_version(),
        kind: "ops_run_report_v1".to_string(),
        command: command.to_string(),
        run_id: run_id.clone(),
        repo_root: repo_root.display().to_string(),
        ops_root: ops_root.display().to_string(),
        profile: common.profile.clone(),
        suite,
        status: status.to_string(),
        exit_code,
        checks: Vec::new(),
        warnings,
        errors,
        capabilities,
        summary,
        rows,
    }
}

pub(crate) fn render_ops_human(report: &OpsRunReport) -> String {
    let mut lines = vec![
        format!("ops {} [{}]", report.command, report.status),
        format!("run_id={}", report.run_id),
        format!(
            "errors={} warnings={}",
            report.summary.get("errors").copied().unwrap_or(0),
            report.summary.get("warnings").copied().unwrap_or(0)
        ),
    ];
    let mut errs = report.errors.clone();
    errs.sort();
    for err in errs {
        lines.push(format!("E {err}"));
    }
    let mut warns = report.warnings.clone();
    warns.sort();
    for warn in warns {
        lines.push(format!("W {warn}"));
    }
    lines.join("\n")
}

pub(crate) fn sha256_hex(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn run_ops_checks(
    common: &OpsCommonArgs,
    suite: &str,
    include_internal: bool,
    include_slow: bool,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let selectors = parse_selectors(
        Some(suite.to_string()),
        Some(DomainArg::Ops),
        None,
        None,
        include_internal,
        include_slow,
    )?;
    let request = RunRequest {
        repo_root: repo_root.clone(),
        domain: Some(DomainId::Ops),
        capabilities: Capabilities::deny_all(),
        artifacts_root: Some(
            common
                .artifacts_root
                .clone()
                .unwrap_or_else(|| repo_root.join("artifacts")),
        ),
        run_id: Some(run_id_or_default(common.run_id.clone())?),
        command: Some(format!("bijux dev atlas ops {suite}")),
    };
    let report = run_checks(
        &RealProcessRunner,
        &RealFs,
        &request,
        &selectors,
        &RunOptions {
            fail_fast: common.fail_fast,
            max_failures: common.max_failures,
        },
    )?;
    let rendered = match common.format {
        FormatArg::Text => render_text_with_durations(&report, 10),
        FormatArg::Json => render_json(&report)?,
        FormatArg::Jsonl => render_jsonl(&report)?,
    };
    write_output_if_requested(common.out.clone(), &rendered)?;
    Ok((rendered, exit_code_for_report(&report)))
}

pub(crate) fn verify_tools_snapshot(
    allow_subprocess: bool,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    if !allow_subprocess {
        return Ok(serde_json::json!({
            "enabled": false,
            "text": "tool verification skipped (pass --allow-subprocess)",
            "missing_required": [],
            "rows": []
        }));
    }
    let process = OpsProcess::new(true);
    let mut rows = Vec::new();
    let mut missing_required = Vec::new();
    for (name, definition) in tool_definitions_sorted(inventory) {
        let mut row = process
            .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
            .map_err(|e| e.to_stable_message())?;
        row["required"] = serde_json::Value::Bool(definition.required);
        if definition.required && row["installed"] != serde_json::Value::Bool(true) {
            missing_required.push(name.clone());
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(serde_json::json!({
        "enabled": true,
        "text": if missing_required.is_empty() { "all required tools available" } else { "missing required tools" },
        "missing_required": missing_required,
        "rows": rows
    }))
}

pub(crate) fn render_ops_validation_output(
    common: &OpsCommonArgs,
    mode: &str,
    inventory_errors: &[String],
    checks_rendered: &str,
    checks_exit: i32,
    summary: serde_json::Value,
) -> Result<(String, i32), String> {
    let inventory_error_count = inventory_errors.len();
    let checks_error_count = if checks_exit == 0 { 0 } else { 1 };
    let error_count = inventory_error_count + checks_error_count;
    let status = if error_count == 0 { "ok" } else { "failed" };
    let strict_failed = common.strict && error_count > 0;
    let exit = if strict_failed || checks_exit != 0 || inventory_error_count > 0 {
        1
    } else {
        0
    };
    let text = format!(
        "ops {mode}: status={status} inventory_errors={inventory_error_count} checks_exit={checks_exit}"
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "mode": mode,
        "status": status,
        "text": text,
        "rows": [{
            "inventory_errors": inventory_errors,
            "checks_exit": checks_exit,
            "checks_output": checks_rendered,
            "inventory_summary": summary
        }],
        "summary": {
            "total": 1,
            "errors": error_count,
            "warnings": 0
        }
    });
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit))
}

pub(crate) fn ops_pins_check_payload(
    common: &OpsCommonArgs,
    repo_root: &Path,
) -> Result<(serde_json::Value, i32), String> {
    let ops_root =
        resolve_ops_root(repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut errors = Vec::new();
    if let Err(err) =
        bijux_dev_atlas_core::ops_inventory::OpsInventory::load_and_validate(&ops_root)
    {
        errors.push(err);
    }
    let pins = load_stack_pins(repo_root).map_err(|e| e.to_stable_message())?;
    errors.extend(validate_pins_completeness(repo_root, &pins).map_err(|e| e.to_stable_message())?);
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": status,
        "text": if errors.is_empty() { "ops pins check passed" } else { "ops pins check failed" },
        "rows": [{
            "pins_path": "ops/stack/pins.toml",
            "errors": errors
        }],
        "summary": {"total": 1, "errors": if status == "ok" {0} else {1}, "warnings": 0}
    });
    Ok((payload, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn load_toolchain_inventory_for_ops(
    ops_root: &Path,
) -> Result<ToolchainInventory, OpsCommandError> {
    load_toolchain_inventory(ops_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops_support::StackManifestProfile;

    #[test]
    fn tools_manifest_parses() {
        let root = tempfile::tempdir().expect("tempdir");
        let tools_dir = root.path().join("ops/tools");
        std::fs::create_dir_all(&tools_dir).expect("mkdir");
        std::fs::write(
            tools_dir.join("tools.toml"),
            "[[tools]]\nname=\"helm\"\nrequired=true\nversion_regex=\"(\\\\d+\\\\.\\\\d+\\\\.\\\\d+)\"\nprobe_argv=[\"version\",\"--short\"]\n",
        )
        .expect("write");
        let parsed = load_tools_manifest(root.path()).expect("parse");
        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].name, "helm");
    }

    #[test]
    fn pins_validation_rejects_latest_tag() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack")).expect("mkdir stack");
        std::fs::create_dir_all(root.path().join("ops/k8s/charts/bijux-atlas"))
            .expect("mkdir chart");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        std::fs::write(
            root.path().join("ops/stack/version-manifest.json"),
            "{\"schema_version\":1,\"redis\":\"redis:latest\"}",
        )
        .expect("write manifest");
        std::fs::write(
            root.path().join("ops/k8s/charts/bijux-atlas/values.yaml"),
            "image: redis:latest\n",
        )
        .expect("write values");
        std::fs::write(
            root.path()
                .join("ops/k8s/charts/bijux-atlas/values-offline.yaml"),
            "image: redis:latest\n",
        )
        .expect("write values offline");
        std::fs::write(
            root.path().join("ops/inventory/contracts.json"),
            "{\"contracts\":[{\"path\":\"ops/tools/tools.toml\"},{\"path\":\"ops/stack/pins.toml\"}]}",
        )
        .expect("write contracts");
        let pins = StackPinsToml {
            charts: std::collections::BTreeMap::new(),
            images: std::collections::BTreeMap::from([(
                "redis".to_string(),
                "redis:latest".to_string(),
            )]),
            crds: std::collections::BTreeMap::new(),
        };
        let errors = validate_pins_completeness(root.path(), &pins).expect("validate");
        assert!(errors.iter().any(|e| e.contains("floating tag forbidden")));
    }

    #[test]
    fn stack_manifest_validation_checks_component_order_and_paths() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack/kind")).expect("mkdir");
        std::fs::write(
            root.path().join("ops/stack/kind/cluster.yaml"),
            "kind: Cluster\n",
        )
        .expect("write cluster");
        let manifest = StackManifestToml {
            profiles: std::collections::BTreeMap::from([(
                "kind".to_string(),
                StackManifestProfile {
                    kind_profile: "normal".to_string(),
                    cluster_config: "ops/stack/kind/cluster.yaml".to_string(),
                    namespace: "bijux-atlas".to_string(),
                    components: vec![
                        "ops/stack/redis/redis.yaml".to_string(),
                        "ops/obs/pack/k8s/namespace.yaml".to_string(),
                    ],
                },
            )]),
        };
        let errors = validate_stack_manifest(root.path(), &manifest).expect("validate");
        assert!(errors
            .iter()
            .any(|e| e.contains("components must be lexicographically sorted")));
        assert!(errors.iter().any(|e| e.contains("missing component")));
    }
}
