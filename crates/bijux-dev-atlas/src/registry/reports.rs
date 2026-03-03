// SPDX-License-Identifier: Apache-2.0
//! Typed report registry loading.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const REPORTS_REGISTRY_PATH: &str = "configs/reports/reports.registry.json";
pub const REPORTS_REGISTRY_SCHEMA_PATH: &str =
    "configs/schema/reports/reports.registry.schema.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportRegistryEntry {
    pub report_id: String,
    pub version: u64,
    pub schema_path: String,
    pub example_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportRegistry {
    pub schema_version: u64,
    pub reports: Vec<ReportRegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportCatalogValidation {
    pub report_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportArtifactValidation {
    pub scanned_reports: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportProgress {
    pub total_reports: usize,
    pub missing_example_paths: usize,
    pub missing_schema_files: usize,
    pub rows: Vec<ReportProgressRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportProgressRow {
    pub report_id: String,
    pub missing: Vec<String>,
}

impl ReportRegistry {
    pub fn load(repo_root: &Path) -> Result<Self, String> {
        let path = repo_root.join(REPORTS_REGISTRY_PATH);
        let text = std::fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?;
        let registry: Self = serde_json::from_str(&text)
            .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
        if registry.schema_version != 1 {
            return Err(format!("{} must declare schema_version=1", path.display()));
        }
        for report in &registry.reports {
            if report.report_id.trim().is_empty() {
                return Err(format!("{} contains a blank report_id", path.display()));
            }
            if report.version == 0 {
                return Err(format!(
                    "{} report `{}` must declare version >= 1",
                    path.display(),
                    report.report_id
                ));
            }
        }
        Ok(registry)
    }

    pub fn validate_catalog(repo_root: &Path) -> Result<ReportCatalogValidation, String> {
        let schema_path = repo_root.join(REPORTS_REGISTRY_SCHEMA_PATH);
        let schema_value = read_json(&schema_path)?;
        let registry = Self::load(repo_root)?;
        let mut errors = Vec::new();
        validate_registry_shape(&schema_value, &registry, &mut errors);
        for report in &registry.reports {
            let report_schema_path = repo_root.join(&report.schema_path);
            if !report_schema_path.exists() {
                errors.push(format!(
                    "report `{}` schema path `{}` does not exist",
                    report.report_id, report.schema_path
                ));
                continue;
            }
            match read_json(&report_schema_path) {
                Ok(schema) => validate_schema_entry(report, &schema, &mut errors),
                Err(err) => errors.push(err),
            }
        }
        Ok(ReportCatalogValidation {
            report_count: registry.reports.len(),
            errors,
        })
    }

    pub fn validate_reports_dir(
        repo_root: &Path,
        reports_dir: &Path,
    ) -> Result<ReportArtifactValidation, String> {
        let registry = Self::load(repo_root)?;
        let by_id = registry
            .reports
            .iter()
            .map(|entry| (entry.report_id.as_str(), entry))
            .collect::<BTreeMap<_, _>>();
        let mut errors = Vec::new();
        let mut scanned_reports = 0usize;
        for path in walk_json_files(reports_dir)? {
            scanned_reports += 1;
            let value = match read_json(&path) {
                Ok(value) => value,
                Err(err) => {
                    errors.push(err);
                    continue;
                }
            };
            let Some(report_id) = value.get("report_id").and_then(serde_json::Value::as_str) else {
                errors.push(format!("{} is missing string `report_id`", path.display()));
                continue;
            };
            let Some(entry) = by_id.get(report_id) else {
                errors.push(format!(
                    "{} declares unknown report_id `{report_id}` not listed in {}",
                    path.display(),
                    REPORTS_REGISTRY_PATH
                ));
                continue;
            };
            let version = value
                .get("version")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default();
            if version != entry.version {
                errors.push(format!(
                    "{} declares version {} but registry requires {} for `{}`",
                    path.display(),
                    version,
                    entry.version,
                    report_id
                ));
            }
        }
        Ok(ReportArtifactValidation {
            scanned_reports,
            errors,
        })
    }

    pub fn render_index_markdown(repo_root: &Path) -> Result<String, String> {
        let registry = Self::load(repo_root)?;
        let mut out = String::from("# Report Index\n\n");
        out.push_str("| Report ID | Version | Schema | Example |\n");
        out.push_str("| --- | --- | --- | --- |\n");
        for report in &registry.reports {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` |\n",
                report.report_id, report.version, report.schema_path, report.example_path
            ));
        }
        Ok(out)
    }

    pub fn progress(repo_root: &Path) -> Result<ReportProgress, String> {
        let registry = Self::load(repo_root)?;
        let mut rows = Vec::new();
        let mut missing_example_paths = 0usize;
        let mut missing_schema_files = 0usize;
        for report in &registry.reports {
            let mut missing = Vec::new();
            if report.example_path.trim().is_empty() {
                missing.push("example_path".to_string());
                missing_example_paths += 1;
            }
            if !repo_root.join(&report.schema_path).exists() {
                missing.push("schema_file".to_string());
                missing_schema_files += 1;
            }
            if !missing.is_empty() {
                rows.push(ReportProgressRow {
                    report_id: report.report_id.clone(),
                    missing,
                });
            }
        }
        Ok(ReportProgress {
            total_reports: registry.reports.len(),
            missing_example_paths,
            missing_schema_files,
            rows,
        })
    }
}

fn validate_registry_shape(
    schema_value: &serde_json::Value,
    registry: &ReportRegistry,
    errors: &mut Vec<String>,
) {
    let expected_version = schema_value
        .get("properties")
        .and_then(|value| value.get("schema_version"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_u64);
    if expected_version != Some(registry.schema_version) {
        errors.push(format!(
            "{} expects schema_version {:?} but registry declares {}",
            REPORTS_REGISTRY_SCHEMA_PATH, expected_version, registry.schema_version
        ));
    }
}

fn validate_schema_entry(
    report: &ReportRegistryEntry,
    schema: &serde_json::Value,
    errors: &mut Vec<String>,
) {
    let declared_report_id = schema
        .get("properties")
        .and_then(|value| value.get("report_id"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_str);
    if declared_report_id != Some(report.report_id.as_str()) {
        errors.push(format!(
            "schema `{}` declares report_id {:?} but registry expects `{}`",
            report.schema_path, declared_report_id, report.report_id
        ));
    }
    let declared_version = schema
        .get("properties")
        .and_then(|value| value.get("version"))
        .and_then(|value| value.get("const"))
        .and_then(serde_json::Value::as_u64);
    if declared_version != Some(report.version) {
        errors.push(format!(
            "schema `{}` declares version {:?} but registry expects {}",
            report.schema_path, declared_version, report.version
        ));
    }
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn walk_json_files(root: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files = Vec::new();
    collect_json_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_json_files(root: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    if root.is_file() {
        if root.extension().and_then(|ext| ext.to_str()) == Some("json") {
            files.push(root.to_path_buf());
        }
        return Ok(());
    }
    let entries =
        fs::read_dir(root).map_err(|err| format!("read {} failed: {err}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read {} failed: {err}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            files.push(path);
        }
    }
    Ok(())
}
