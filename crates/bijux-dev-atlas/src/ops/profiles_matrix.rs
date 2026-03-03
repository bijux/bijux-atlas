// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ops::helm_env;

#[path = "profiles_matrix/validation_support.rs"]
mod validation_support;
use validation_support::{
    dataset_validation_status, kubeconform_profile, template_profile, template_profile_output,
    validate_values_file,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ToolInvocationReport {
    pub binary: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub status: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StatusReport {
    pub status: String,
    pub note: String,
    pub errors: Vec<String>,
    pub event: ToolInvocationReport,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProfileMatrixRow {
    pub profile: String,
    pub values_file: String,
    pub helm_template: StatusReport,
    pub values_schema: StatusReport,
    pub dataset_validation: StatusReport,
    pub kubeconform: StatusReport,
    pub rendered_resources: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProfileMatrixSummary {
    pub total: usize,
    pub helm_failures: usize,
    pub schema_failures: usize,
    pub dataset_failures: usize,
    pub kubeconform_failures: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ToolVersionRow {
    pub binary: String,
    pub probe_argv: Vec<String>,
    pub declared: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProfilesMatrixInputs {
    pub chart_dir: String,
    pub values_root: String,
    pub schema_path: String,
    pub dataset_manifest_path: String,
    pub profile_selector: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProfilesMatrixReport {
    pub report_id: String,
    pub version: u64,
    pub schema_version: u64,
    pub kind: String,
    pub inputs: ProfilesMatrixInputs,
    pub tooling: Vec<ToolVersionRow>,
    pub rows: Vec<ProfileMatrixRow>,
    pub summary: ProfileMatrixSummary,
}

#[derive(Debug, Deserialize)]
struct ToolchainInventory {
    tools: BTreeMap<String, ToolDefinition>,
}

#[derive(Debug, Deserialize)]
struct ToolDefinition {
    probe_argv: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct InstallMatrixDoc {
    profiles: Vec<InstallMatrixProfile>,
}

#[derive(Debug, Deserialize, Clone)]
struct InstallMatrixProfile {
    name: String,
    values_file: String,
}

#[derive(Debug, Deserialize)]
struct RolloutSafetyDoc {
    profiles: Vec<RolloutSafetyProfile>,
}

#[derive(Debug, Deserialize)]
struct RolloutSafetyProfile {
    name: String,
}

#[derive(Debug, Clone)]
pub struct ValidateProfilesOptions {
    pub chart_dir: PathBuf,
    pub values_root: PathBuf,
    pub schema_path: PathBuf,
    pub dataset_manifest_path: PathBuf,
    pub install_matrix_path: PathBuf,
    pub rollout_safety_path: PathBuf,
    pub profile: Option<String>,
    pub profile_set: Option<String>,
    pub timeout_seconds: u64,
    pub run_kubeconform: bool,
}

fn binary_exists(binary: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            let candidate = dir.join(binary);
            candidate.is_file()
        })
    })
}

fn run_with_timeout(
    binary: &str,
    args: &[String],
    cwd: &Path,
    timeout_seconds: u64,
) -> Result<std::process::Output, String> {
    let mut command = Command::new(binary);
    command.current_dir(cwd).args(args);
    command.env_clear();
    for key in [
        "HOME", "PATH", "TMPDIR", "TEMP", "TMP", "USER", "LOGNAME", "SHELL",
    ] {
        if let Ok(value) = std::env::var(key) {
            command.env(key, value);
        }
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|err| format!("failed to start `{binary}`: {err}"))?;
    let timeout = Duration::from_secs(timeout_seconds.max(1));
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|err| format!("failed to collect `{binary}` output: {err}"));
            }
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "`{binary}` timed out after {}s",
                        timeout_seconds.max(1)
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("failed to poll `{binary}`: {err}"));
            }
        }
    }
}

pub fn discover_profiles(values_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut rows = std::fs::read_dir(values_root)
        .map_err(|err| format!("failed to read {}: {err}", values_root.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("yaml"))
        .collect::<Vec<_>>();
    rows.sort();
    Ok(rows)
}

fn load_tooling(repo_root: &Path) -> Result<Vec<ToolVersionRow>, String> {
    let path = repo_root.join("ops/inventory/toolchain.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let inventory: ToolchainInventory = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    let mut rows = Vec::new();
    for binary in ["helm", "kubeconform"] {
        let declared = inventory.tools.contains_key(binary);
        let probe_argv = inventory
            .tools
            .get(binary)
            .map(|row| row.probe_argv.clone())
            .unwrap_or_default();
        rows.push(ToolVersionRow {
            binary: binary.to_string(),
            probe_argv,
            declared,
        });
    }
    Ok(rows)
}

fn load_install_matrix(path: &Path) -> Result<InstallMatrixDoc, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn load_rollout_safety(path: &Path) -> Result<RolloutSafetyDoc, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn stage_kubeconform_manifest(repo_root: &Path, rendered_yaml: &str) -> Result<PathBuf, String> {
    let temp_dir = repo_root.join("artifacts/ops/profile-render-matrix/tmp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|err| format!("failed to create {}: {err}", temp_dir.display()))?;
    let rendered_path = temp_dir.join("rendered.yaml");
    std::fs::write(&rendered_path, rendered_yaml)
        .map_err(|err| format!("failed to write {}: {err}", rendered_path.display()))?;
    Ok(rendered_path)
}

fn selected_profiles(
    values_root: &Path,
    matrix: &InstallMatrixDoc,
    rollout_safety: &RolloutSafetyDoc,
    profile: Option<&str>,
    profile_set: Option<&str>,
) -> Result<Vec<InstallMatrixProfile>, String> {
    let _ = discover_profiles(values_root)?;
    let mut rows = matrix.profiles.clone();
    rows.sort_by(|left, right| left.name.cmp(&right.name));
    if let Some(name) = profile {
        let selected = rows
            .into_iter()
            .filter(|row| row.name == name)
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return Err(format!(
                "profile `{name}` is not declared in install matrix"
            ));
        }
        return Ok(selected);
    }
    if let Some(name) = profile_set {
        if name != "rollout-safety" {
            return Err(format!("unknown profile set `{name}`"));
        }
        let mut selected = Vec::new();
        for required_name in rollout_safety.profiles.iter().map(|row| row.name.as_str()) {
            let Some(row) = rows.iter().find(|row| row.name == required_name).cloned() else {
                return Err(format!(
                    "rollout safety contract references missing install-matrix profile `{required_name}`"
                ));
            };
            selected.push(row);
        }
        selected.sort_by(|left, right| left.name.cmp(&right.name));
        return Ok(selected);
    }
    Ok(rows)
}

fn compile_values_schema(schema_path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(schema_path)
        .map_err(|err| format!("failed to read {}: {err}", schema_path.display()))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", schema_path.display()))
}

fn load_dataset_manifest_ids(manifest_path: &Path) -> Result<BTreeMap<String, String>, String> {
    let text = std::fs::read_to_string(manifest_path)
        .map_err(|err| format!("failed to read {}: {err}", manifest_path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", manifest_path.display()))?;
    if json.get("schema_version").and_then(|value| value.as_u64()) != Some(1) {
        return Err(format!(
            "{} must declare schema_version=1",
            manifest_path.display()
        ));
    }
    let datasets = json
        .get("datasets")
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("{} must declare datasets array", manifest_path.display()))?;
    let mut rows = BTreeMap::new();
    for dataset in datasets {
        let dataset_id = dataset
            .get("id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                format!(
                    "{} datasets entries must include string id",
                    manifest_path.display()
                )
            })?;
        let dataset_name = dataset
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or(dataset_id);
        rows.insert(dataset_id.to_string(), dataset_name.to_string());
    }
    Ok(rows)
}

fn values_yaml_to_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    serde_json::to_value(yaml)
        .map_err(|err| format!("failed to convert {} to json: {err}", path.display()))
}

fn merge_values(base: &mut serde_json::Value, overlay: serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(base_obj), serde_json::Value::Object(overlay_obj)) => {
            for (key, overlay_value) in overlay_obj {
                match base_obj.get_mut(&key) {
                    Some(base_value) => merge_values(base_value, overlay_value),
                    None => {
                        base_obj.insert(key, overlay_value);
                    }
                }
            }
        }
        (base_slot, overlay_value) => *base_slot = overlay_value,
    }
}

pub fn build_report(
    rows: Vec<ProfileMatrixRow>,
    inputs: ProfilesMatrixInputs,
    tooling: Vec<ToolVersionRow>,
) -> ProfilesMatrixReport {
    let summary = ProfileMatrixSummary {
        total: rows.len(),
        helm_failures: rows
            .iter()
            .filter(|row| row.helm_template.status == "fail")
            .count(),
        schema_failures: rows
            .iter()
            .filter(|row| row.values_schema.status == "fail")
            .count(),
        dataset_failures: rows
            .iter()
            .filter(|row| row.dataset_validation.status == "fail")
            .count(),
        kubeconform_failures: rows
            .iter()
            .filter(|row| row.kubeconform.status == "fail")
            .count(),
    };
    ProfilesMatrixReport {
        report_id: "ops-profiles".to_string(),
        version: 1,
        schema_version: 1,
        kind: "ops_profiles_matrix".to_string(),
        inputs,
        tooling,
        rows,
        summary,
    }
}

pub fn validate_report_schema_file(schema_path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(schema_path)
        .map_err(|err| format!("failed to read {}: {err}", schema_path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", schema_path.display()))?;
    let required = json
        .get("required")
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("{} must declare required array", schema_path.display()))?;
    for field in [
        "report_id",
        "version",
        "schema_version",
        "kind",
        "inputs",
        "tooling",
        "rows",
        "summary",
    ] {
        if !required.iter().any(|value| value.as_str() == Some(field)) {
            return Err(format!("{} must require `{field}`", schema_path.display()));
        }
    }
    Ok(())
}

pub fn validate_report_value(report: &serde_json::Value, schema_path: &Path) -> Result<(), String> {
    validate_report_schema_file(schema_path)?;
    let obj = report
        .as_object()
        .ok_or_else(|| "ops-profiles report must be a JSON object".to_string())?;
    if obj.get("report_id").and_then(|value| value.as_str()) != Some("ops-profiles") {
        return Err("ops-profiles report must declare report_id=ops-profiles".to_string());
    }
    if obj.get("version").and_then(|value| value.as_u64()) != Some(1) {
        return Err("ops-profiles report must declare version=1".to_string());
    }
    if obj.get("schema_version").and_then(|value| value.as_u64()) != Some(1) {
        return Err("ops-profiles report must declare schema_version=1".to_string());
    }
    if obj.get("kind").and_then(|value| value.as_str()) != Some("ops_profiles_matrix") {
        return Err("ops-profiles report must declare kind=ops_profiles_matrix".to_string());
    }
    if obj.get("rows").and_then(|value| value.as_array()).is_none() {
        return Err("ops-profiles report rows must be an array".to_string());
    }
    if obj
        .get("tooling")
        .and_then(|value| value.as_array())
        .is_none()
    {
        return Err("ops-profiles report tooling must be an array".to_string());
    }
    Ok(())
}

pub fn validate_profiles(
    repo_root: &Path,
    options: &ValidateProfilesOptions,
) -> Result<ProfilesMatrixReport, String> {
    let helm_binary = helm_env::resolve_helm_binary_from_inventory(repo_root)?;
    let tooling = load_tooling(repo_root)?;
    let matrix = load_install_matrix(&options.install_matrix_path)?;
    let rollout_safety = load_rollout_safety(&options.rollout_safety_path)?;
    let validator = compile_values_schema(&options.schema_path)?;
    let dataset_manifest_ids = load_dataset_manifest_ids(&options.dataset_manifest_path)?;
    let base_values = values_yaml_to_json(&options.chart_dir.join("values.yaml"))?;
    let selector_label = if let Some(name) = &options.profile {
        format!("single:{name}")
    } else if let Some(name) = &options.profile_set {
        format!("set:{name}")
    } else {
        "all".to_string()
    };
    let mut rows = Vec::new();
    for profile in selected_profiles(
        &options.values_root,
        &matrix,
        &rollout_safety,
        options.profile.as_deref(),
        options.profile_set.as_deref(),
    )? {
        let values_path = repo_root.join(&profile.values_file);
        if !values_path.is_file() {
            return Err(format!(
                "install matrix profile `{}` references missing values file `{}`",
                profile.name, profile.values_file
            ));
        }
        let mut merged_values = base_values.clone();
        merge_values(&mut merged_values, values_yaml_to_json(&values_path)?);
        let schema_errors = validate_values_file(&validator, &merged_values)?;
        let values_schema = if schema_errors.is_empty() {
            StatusReport {
                status: "pass".to_string(),
                note: "values schema validated".to_string(),
                errors: Vec::new(),
                event: ToolInvocationReport {
                    binary: "values.schema.json".to_string(),
                    args: vec![options.schema_path.display().to_string()],
                    cwd: repo_root.display().to_string(),
                    status: "pass".to_string(),
                    stderr: String::new(),
                },
            }
        } else {
            StatusReport {
                status: "fail".to_string(),
                note: "values schema failure".to_string(),
                errors: schema_errors.clone(),
                event: ToolInvocationReport {
                    binary: "values.schema.json".to_string(),
                    args: vec![options.schema_path.display().to_string()],
                    cwd: repo_root.display().to_string(),
                    status: "fail".to_string(),
                    stderr: String::new(),
                },
            }
        };
        let dataset_validation = dataset_validation_status(
            repo_root,
            &options.dataset_manifest_path,
            &dataset_manifest_ids,
            &merged_values,
        );

        let helm_template = template_profile(
            repo_root,
            &helm_binary,
            &options.chart_dir,
            &values_path,
            &profile.name,
            options.timeout_seconds,
        );

        let (kubeconform, rendered_resources) = if helm_template.status == "pass" {
            match template_profile_output(
                repo_root,
                &helm_binary,
                &options.chart_dir,
                &values_path,
                &profile.name,
                options.timeout_seconds,
            ) {
                Ok((rendered_yaml, _stderr)) => {
                    let rendered_resources =
                        serde_yaml::Deserializer::from_str(&rendered_yaml).count();
                    let staged_manifest = stage_kubeconform_manifest(repo_root, &rendered_yaml);
                    (
                        kubeconform_profile(repo_root, staged_manifest, options.timeout_seconds, options.run_kubeconform),
                        rendered_resources,
                    )
                }
                Err(message) => (
                    StatusReport {
                        status: "fail".to_string(),
                        note: "helm render replay failure".to_string(),
                        errors: vec![message.clone()],
                        event: ToolInvocationReport {
                            binary: helm_binary.clone(),
                            args: vec!["template".to_string()],
                            cwd: repo_root.display().to_string(),
                            status: "fail".to_string(),
                            stderr: message,
                        },
                    },
                    0,
                ),
            }
        } else {
            (
                StatusReport {
                    status: "skipped".to_string(),
                    note: "kubeconform skipped because helm template failed".to_string(),
                    errors: Vec::new(),
                    event: ToolInvocationReport {
                        binary: "kubeconform".to_string(),
                        args: vec![
                            "-strict".to_string(),
                            "-summary".to_string(),
                            "-ignore-missing-schemas".to_string(),
                            "<rendered-manifest>".to_string(),
                        ],
                        cwd: repo_root.display().to_string(),
                        status: "skipped".to_string(),
                        stderr: String::new(),
                    },
                },
                0,
            )
        };

        rows.push(ProfileMatrixRow {
            profile: profile.name,
            values_file: profile.values_file,
            helm_template,
            values_schema,
            dataset_validation,
            kubeconform,
            rendered_resources,
        });
    }
    rows.sort_by(|left, right| left.profile.cmp(&right.profile));
    Ok(build_report(
        rows,
        ProfilesMatrixInputs {
            chart_dir: options.chart_dir.display().to_string(),
            values_root: options.values_root.display().to_string(),
            schema_path: options.schema_path.display().to_string(),
            dataset_manifest_path: options.dataset_manifest_path.display().to_string(),
            profile_selector: selector_label,
        },
        tooling,
    ))
}

#[cfg(test)]
mod tests;
