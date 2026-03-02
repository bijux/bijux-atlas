// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    pub schema_version: u64,
    pub kind: String,
    pub inputs: HelmEnvInputs,
    pub env_keys: Vec<String>,
    pub config_maps: Vec<ConfigMapEnvRow>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelmEnvInputs {
    pub chart_dir: String,
    pub values_files: Vec<String>,
    pub release_name: String,
    pub helm_binary: String,
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

pub fn render_chart(
    repo_root: &Path,
    helm_binary: &str,
    chart_dir: &Path,
    values_files: &[PathBuf],
    release_name: &str,
) -> Result<Vec<YamlDoc>, String> {
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
    for values_file in values_files {
        command.arg("-f").arg(values_file);
    }
    command.env_clear();
    for key in [
        "HOME", "PATH", "TMPDIR", "TEMP", "TMP", "USER", "LOGNAME", "SHELL",
    ] {
        if let Ok(value) = std::env::var(key) {
            command.env(key, value);
        }
    }
    let output = command
        .output()
        .map_err(|err| format!("failed to start `{helm_binary}`: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "helm template failed for `{}`: {}",
            chart_dir.display(),
            stderr
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_yaml_stream(&stdout, &format!("{} stdout", helm_binary))
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

pub fn build_report(
    chart_dir: &Path,
    values_files: &[PathBuf],
    release_name: &str,
    helm_binary: &str,
    env_keys: &BTreeSet<String>,
    config_maps: &[ConfigMapEnvRow],
    include_names: bool,
) -> HelmEnvReport {
    let mut config_maps = config_maps.to_vec();
    if !include_names {
        config_maps.clear();
    }
    HelmEnvReport {
        schema_version: 1,
        kind: "ops_helm_env".to_string(),
        inputs: HelmEnvInputs {
            chart_dir: chart_dir.display().to_string(),
            values_files: values_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            release_name: release_name.to_string(),
            helm_binary: helm_binary.to_string(),
        },
        env_keys: env_keys.iter().cloned().collect(),
        config_maps,
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
            Path::new("ops/k8s/charts/bijux-atlas"),
            &[PathBuf::from("ops/k8s/charts/bijux-atlas/values.yaml")],
            "bijux-atlas",
            "helm",
            &keys,
            &rows,
            false,
        );
        assert!(report.config_maps.is_empty());
        assert_eq!(report.env_keys, vec!["ATLAS_ONE".to_string()]);
    }
}
