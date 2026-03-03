// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct YamlDoc {
    pub source: String,
    pub document_index: usize,
    pub value: serde_yaml::Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigMapEnvRow {
    pub config_map_name: String,
    pub env_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelmEnvReport {
    pub report_id: String,
    pub version: u64,
    pub schema_version: u64,
    pub kind: String,
    pub inputs: HelmEnvInputs,
    pub env_keys: Vec<String>,
    pub config_maps: Vec<ConfigMapEnvRow>,
    pub helm: HelmInvocationReport,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelmEnvInputs {
    pub chart_dir: String,
    pub values_files: Vec<String>,
    pub release_name: String,
    pub helm_binary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelmInvocationReport {
    pub status: String,
    pub debug_enabled: bool,
    pub timeout_seconds: u64,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct RenderChartOptions {
    pub set_overrides: Vec<String>,
    pub timeout_seconds: u64,
    pub debug: bool,
}

#[derive(Debug, Clone)]
pub struct RenderedChart {
    pub yaml_docs: Vec<YamlDoc>,
    pub stderr: String,
    pub debug_enabled: bool,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct ToolchainInventory {
    tools: std::collections::BTreeMap<String, serde_json::Value>,
}

pub fn resolve_helm_binary_from_inventory(repo_root: &Path) -> Result<String, String> {
    let inventory_path = repo_root.join("ops/inventory/toolchain.json");
    let inventory_text = std::fs::read_to_string(&inventory_path)
        .map_err(|err| format!("failed to read {}: {err}", inventory_path.display()))?;
    let inventory: ToolchainInventory = serde_json::from_str(&inventory_text)
        .map_err(|err| format!("failed to parse {}: {err}", inventory_path.display()))?;
    if inventory.tools.contains_key("helm") {
        Ok("helm".to_string())
    } else {
        Err(format!(
            "ops toolchain inventory {} must declare tool `helm`",
            inventory_path.display()
        ))
    }
}

pub fn parse_yaml_stream(text: &str, source: &str) -> Result<Vec<YamlDoc>, String> {
    let mut docs = Vec::new();
    for (index, document) in serde_yaml::Deserializer::from_str(text).enumerate() {
        match serde_yaml::Value::deserialize(document) {
            Ok(value) => docs.push(YamlDoc {
                source: source.to_string(),
                document_index: index,
                value,
            }),
            Err(err) => {
                let location = err
                    .location()
                    .map(|loc| format!(" line {} column {}", loc.line(), loc.column()))
                    .unwrap_or_default();
                return Err(format!(
                    "{} document {}{}: {}",
                    source,
                    index + 1,
                    location,
                    err
                ));
            }
        }
    }
    Ok(docs)
}

fn run_helm_with_timeout(
    command: &mut Command,
    timeout_seconds: u64,
) -> Result<std::process::Output, String> {
    let binary = command.get_program().to_string_lossy().to_string();
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to start `{binary}`: {err}"))?;
    let timeout = Duration::from_secs(timeout_seconds.max(1));
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|err| format!("failed to collect helm output: {err}"));
            }
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "helm template timed out after {}s",
                        timeout_seconds.max(1)
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("failed to poll helm process: {err}"));
            }
        }
    }
}

pub fn render_chart(
    repo_root: &Path,
    helm_binary: &str,
    chart_dir: &Path,
    values_files: &[PathBuf],
    release_name: &str,
) -> Result<RenderedChart, String> {
    render_chart_with_options(
        repo_root,
        helm_binary,
        chart_dir,
        values_files,
        release_name,
        &RenderChartOptions {
            set_overrides: Vec::new(),
            timeout_seconds: 30,
            debug: false,
        },
    )
}

pub fn render_chart_with_options(
    repo_root: &Path,
    helm_binary: &str,
    chart_dir: &Path,
    values_files: &[PathBuf],
    release_name: &str,
    options: &RenderChartOptions,
) -> Result<RenderedChart, String> {
    if !chart_dir.exists() {
        return Err(format!(
            "chart path does not exist: {}",
            chart_dir.display()
        ));
    }
    for values_file in values_files {
        if !values_file.exists() {
            return Err(format!(
                "values file does not exist: {}",
                values_file.display()
            ));
        }
    }

    let mut command = Command::new(helm_binary);
    command
        .current_dir(repo_root)
        .arg("template")
        .arg(release_name)
        .arg(chart_dir);
    if options.debug {
        command.arg("--debug");
    }
    for values_file in values_files {
        command.arg("-f").arg(values_file);
    }
    for override_value in &options.set_overrides {
        command.arg("--set").arg(override_value);
    }
    command.env_clear();
    for key in [
        "HOME", "PATH", "TMPDIR", "TEMP", "TMP", "USER", "LOGNAME", "SHELL",
    ] {
        if let Ok(value) = std::env::var(key) {
            command.env(key, value);
        }
    }
    let output = run_helm_with_timeout(&mut command, options.timeout_seconds)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "helm template failed for `{}`: {}",
            chart_dir.display(),
            stderr
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let yaml_docs = parse_yaml_stream(&stdout, &format!("{} stdout", helm_binary))?;
    Ok(RenderedChart {
        yaml_docs,
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        debug_enabled: options.debug,
        timeout_seconds: options.timeout_seconds.max(1),
    })
}

fn config_map_name(doc: &YamlDoc) -> Option<String> {
    let mapping = doc.value.as_mapping()?;
    let kind = mapping
        .get(serde_yaml::Value::String("kind".to_string()))?
        .as_str()?;
    if kind != "ConfigMap" {
        return None;
    }
    let metadata = mapping
        .get(serde_yaml::Value::String("metadata".to_string()))?
        .as_mapping()?;
    metadata
        .get(serde_yaml::Value::String("name".to_string()))
        .and_then(serde_yaml::Value::as_str)
        .map(str::to_string)
}

fn release_name_matches(config_map_name: &str, release_name: &str) -> bool {
    config_map_name == release_name || config_map_name.starts_with(&format!("{release_name}-"))
}

pub fn extract_configmap_rows(yaml_docs: &[YamlDoc], release_name: &str) -> Vec<ConfigMapEnvRow> {
    let mut rows = Vec::new();
    for doc in yaml_docs {
        let Some(name) = config_map_name(doc) else {
            continue;
        };
        if !release_name_matches(&name, release_name) {
            continue;
        }
        let Some(mapping) = doc.value.as_mapping() else {
            continue;
        };
        let Some(data) = mapping
            .get(serde_yaml::Value::String("data".to_string()))
            .and_then(serde_yaml::Value::as_mapping)
        else {
            continue;
        };
        let mut env_keys = data
            .keys()
            .filter_map(serde_yaml::Value::as_str)
            .map(str::trim)
            .filter(|key| {
                (key.starts_with("ATLAS_") || key.starts_with("BIJUX_"))
                    && key.len() > "ATLAS_".len()
            })
            .map(str::to_string)
            .collect::<Vec<_>>();
        env_keys.sort();
        env_keys.dedup();
        if env_keys.is_empty() {
            continue;
        }
        rows.push(ConfigMapEnvRow {
            config_map_name: name,
            env_keys,
        });
    }
    rows.sort_by(|left, right| left.config_map_name.cmp(&right.config_map_name));
    rows
}

pub fn extract_configmap_env_keys(yaml_docs: &[YamlDoc], release_name: &str) -> BTreeSet<String> {
    extract_configmap_rows(yaml_docs, release_name)
        .into_iter()
        .flat_map(|row| row.env_keys.into_iter())
        .collect()
}

pub fn build_inputs(
    chart_dir: &Path,
    values_files: &[PathBuf],
    release_name: &str,
    helm_binary: &str,
) -> HelmEnvInputs {
    HelmEnvInputs {
        chart_dir: chart_dir.display().to_string(),
        values_files: values_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        release_name: release_name.to_string(),
        helm_binary: helm_binary.to_string(),
    }
}

pub fn build_report(
    inputs: HelmEnvInputs,
    env_keys: &BTreeSet<String>,
    config_maps: &[ConfigMapEnvRow],
    include_names: bool,
    helm: HelmInvocationReport,
) -> HelmEnvReport {
    let mut config_maps = config_maps.to_vec();
    if !include_names {
        config_maps.clear();
    }
    HelmEnvReport {
        report_id: "helm-env".to_string(),
        version: 1,
        schema_version: 1,
        kind: "ops_helm_env".to_string(),
        inputs,
        env_keys: env_keys.iter().cloned().collect(),
        config_maps,
        helm,
    }
}

pub fn validate_report_schema_file(schema_path: &Path) -> Result<(), String> {
    let schema_text = std::fs::read_to_string(schema_path)
        .map_err(|err| format!("failed to read {}: {err}", schema_path.display()))?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_text)
        .map_err(|err| format!("failed to parse {}: {err}", schema_path.display()))?;
    let required = schema_json
        .get("required")
        .and_then(|value| value.as_array())
        .ok_or_else(|| format!("{} must declare required array", schema_path.display()))?;
    for field in [
        "report_id",
        "version",
        "schema_version",
        "kind",
        "inputs",
        "env_keys",
        "config_maps",
        "helm",
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
        .ok_or_else(|| "helm-env report must be a JSON object".to_string())?;
    if obj.get("report_id").and_then(|value| value.as_str()) != Some("helm-env") {
        return Err("helm-env report must declare report_id=helm-env".to_string());
    }
    if obj.get("version").and_then(|value| value.as_u64()) != Some(1) {
        return Err("helm-env report must declare version=1".to_string());
    }
    if obj.get("schema_version").and_then(|value| value.as_u64()) != Some(1) {
        return Err("helm-env report must declare schema_version=1".to_string());
    }
    if obj.get("kind").and_then(|value| value.as_str()) != Some("ops_helm_env") {
        return Err("helm-env report must declare kind=ops_helm_env".to_string());
    }
    let inputs = obj
        .get("inputs")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "helm-env report must include object inputs".to_string())?;
    for field in ["chart_dir", "release_name", "helm_binary"] {
        if inputs
            .get(field)
            .and_then(|value| value.as_str())
            .is_none_or(|value| value.is_empty())
        {
            return Err(format!(
                "helm-env report inputs.{field} must be a non-empty string"
            ));
        }
    }
    if inputs
        .get("values_files")
        .and_then(|value| value.as_array())
        .is_none()
    {
        return Err("helm-env report inputs.values_files must be an array".to_string());
    }
    if obj
        .get("env_keys")
        .and_then(|value| value.as_array())
        .is_none()
    {
        return Err("helm-env report env_keys must be an array".to_string());
    }
    if obj
        .get("config_maps")
        .and_then(|value| value.as_array())
        .is_none()
    {
        return Err("helm-env report config_maps must be an array".to_string());
    }
    let helm = obj
        .get("helm")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "helm-env report must include object helm".to_string())?;
    if helm
        .get("status")
        .and_then(|value| value.as_str())
        .is_none_or(|value| value != "ok" && value != "error")
    {
        return Err("helm-env report helm.status must be `ok` or `error`".to_string());
    }
    if helm
        .get("timeout_seconds")
        .and_then(|value| value.as_u64())
        .is_none()
    {
        return Err("helm-env report helm.timeout_seconds must be an integer".to_string());
    }
    if helm
        .get("debug_enabled")
        .and_then(|value| value.as_bool())
        .is_none()
    {
        return Err("helm-env report helm.debug_enabled must be a boolean".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelmEnvSubsetReport {
    pub report_id: String,
    pub version: u64,
    pub schema_version: u64,
    pub kind: String,
    pub inputs: HelmEnvInputs,
    pub extra: Vec<String>,
    pub missing: Vec<String>,
    pub counts: HelmEnvSubsetCounts,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelmEnvSubsetCounts {
    pub emitted: usize,
    pub allowed: usize,
    pub extra: usize,
    pub missing: usize,
}

pub fn load_allowlist(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    let Some(values) = json.get("allowed_env").and_then(|value| value.as_array()) else {
        return Err(format!("{} must declare allowed_env array", path.display()));
    };
    Ok(values
        .iter()
        .filter_map(|value| value.as_str())
        .map(str::to_string)
        .collect())
}

pub fn build_subset_report(
    env_keys: &BTreeSet<String>,
    allowed_env: &BTreeSet<String>,
    inputs: HelmEnvInputs,
) -> HelmEnvSubsetReport {
    let extra = env_keys
        .difference(allowed_env)
        .cloned()
        .collect::<Vec<_>>();
    let missing = allowed_env
        .difference(env_keys)
        .cloned()
        .collect::<Vec<_>>();
    HelmEnvSubsetReport {
        report_id: "helm-env-subset".to_string(),
        version: 1,
        schema_version: 1,
        kind: "ops_helm_env_subset".to_string(),
        inputs,
        counts: HelmEnvSubsetCounts {
            emitted: env_keys.len(),
            allowed: allowed_env.len(),
            extra: extra.len(),
            missing: missing.len(),
        },
        extra,
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_fixture(yaml: &str) -> Vec<YamlDoc> {
        parse_yaml_stream(yaml, "inline fixture").expect("yaml fixture")
    }

    #[test]
    fn parses_inline_yaml_fixture() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\ndata:\n  ATLAS_ONE: \"1\"\n",
        );
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].source, "inline fixture");
        assert_eq!(docs[0].document_index, 0);
    }

    #[test]
    fn extracts_configmap_data_keys() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\ndata:\n  ATLAS_ONE: \"1\"\n  BIJUX_TWO: \"2\"\n",
        );
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        assert_eq!(
            keys.into_iter().collect::<Vec<_>>(),
            vec!["ATLAS_ONE".to_string(), "BIJUX_TWO".to_string()]
        );
    }

    #[test]
    fn ignores_non_configmap_documents() {
        let docs = parse_fixture(
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: bijux-atlas\nspec: {}\n",
        );
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        assert!(keys.is_empty());
    }

    #[test]
    fn ignores_configmaps_without_data() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\n",
        );
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        assert!(keys.is_empty());
    }

    #[test]
    fn ignores_configmaps_outside_the_release_name_surface() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: unrelated-config\ndata:\n  ATLAS_ONE: \"1\"\n",
        );
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        assert!(keys.is_empty());
    }

    #[test]
    fn filters_only_prefixed_keys() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\ndata:\n  PLAIN_KEY: \"1\"\n  ATLAS_ONE: \"1\"\n  BIJUX_TWO: \"2\"\n",
        );
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        assert_eq!(
            keys.into_iter().collect::<Vec<_>>(),
            vec!["ATLAS_ONE".to_string(), "BIJUX_TWO".to_string()]
        );
    }

    #[test]
    fn supports_multi_document_yaml_stream() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\ndata:\n  ATLAS_ONE: \"1\"\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-extra\ndata:\n  BIJUX_TWO: \"2\"\n",
        );
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        assert_eq!(
            keys.into_iter().collect::<Vec<_>>(),
            vec!["ATLAS_ONE".to_string(), "BIJUX_TWO".to_string()]
        );
    }

    #[test]
    fn handles_non_string_data_values_without_panicking() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\ndata:\n  ATLAS_INT: 1\n  BIJUX_BOOL: true\n  NON_PREFIXED: 3\n",
        );
        let rows = extract_configmap_rows(&docs, "bijux-atlas");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].env_keys,
            vec!["ATLAS_INT".to_string(), "BIJUX_BOOL".to_string()]
        );
    }

    #[test]
    fn reports_yaml_parse_error_with_document_and_line_context() {
        let error = parse_yaml_stream(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: demo\n: broken\n",
            "inline fixture",
        )
        .expect_err("parse failure");
        assert!(error.contains("inline fixture document 1"));
        assert!(error.contains("line"));
        assert!(error.contains("column"));
    }

    #[test]
    fn omits_config_map_names_when_include_names_is_disabled() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\ndata:\n  ATLAS_ONE: \"1\"\n",
        );
        let rows = extract_configmap_rows(&docs, "bijux-atlas");
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        let report = build_report(
            build_inputs(
                Path::new("ops/k8s/charts/bijux-atlas"),
                &[PathBuf::from("ops/k8s/charts/bijux-atlas/values.yaml")],
                "bijux-atlas",
                "helm",
            ),
            &keys,
            &rows,
            false,
            HelmInvocationReport {
                status: "ok".to_string(),
                debug_enabled: false,
                timeout_seconds: 30,
                stderr: String::new(),
            },
        );
        assert!(report.config_maps.is_empty());
        assert_eq!(report.env_keys, vec!["ATLAS_ONE".to_string()]);
    }

    #[test]
    fn load_allowlist_reports_missing_file_clearly() {
        let error = load_allowlist(Path::new("missing-allowlist.json")).expect_err("missing file");
        assert!(error.contains("failed to read"));
        assert!(error.contains("missing-allowlist.json"));
    }

    #[test]
    fn render_chart_reports_missing_binary_clearly() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("repo")
            .to_path_buf();
        let error = render_chart(
            &repo_root,
            "definitely-missing-helm-binary",
            &repo_root.join("ops/k8s/charts/bijux-atlas"),
            &[repo_root.join("ops/k8s/charts/bijux-atlas/values.yaml")],
            "bijux-atlas",
        )
        .expect_err("missing helm");
        assert!(error.contains("failed to start"));
    }

    #[test]
    fn render_chart_reports_missing_chart_path_clearly() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("repo")
            .to_path_buf();
        let error = render_chart(
            &repo_root,
            "helm",
            Path::new("missing-chart"),
            &[repo_root.join("ops/k8s/charts/bijux-atlas/values.yaml")],
            "bijux-atlas",
        )
        .expect_err("missing chart");
        assert!(error.contains("chart path does not exist"));
    }

    #[test]
    fn load_allowlist_reports_invalid_json_clearly() {
        let fixture = std::env::temp_dir().join("bijux-helm-env-invalid-allowlist.json");
        std::fs::write(&fixture, "{not-json").expect("write fixture");
        let error = load_allowlist(&fixture).expect_err("invalid json");
        assert!(error.contains("failed to parse"));
        let _ = std::fs::remove_file(&fixture);
    }

    #[test]
    fn validates_report_against_schema_shape() {
        let docs = parse_fixture(
            "apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: bijux-atlas-config\ndata:\n  ATLAS_ONE: \"1\"\n",
        );
        let rows = extract_configmap_rows(&docs, "bijux-atlas");
        let keys = extract_configmap_env_keys(&docs, "bijux-atlas");
        let report = build_report(
            build_inputs(
                Path::new("ops/k8s/charts/bijux-atlas"),
                &[PathBuf::from("ops/k8s/charts/bijux-atlas/values.yaml")],
                "bijux-atlas",
                "helm",
            ),
            &keys,
            &rows,
            true,
            HelmInvocationReport {
                status: "ok".to_string(),
                debug_enabled: false,
                timeout_seconds: 30,
                stderr: String::new(),
            },
        );
        let report_value = serde_json::to_value(report).expect("report json");
        let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace")
            .parent()
            .expect("repo")
            .join("configs/contracts/reports/helm-env.schema.json");
        validate_report_value(&report_value, &schema_path).expect("schema validation");
    }
}
