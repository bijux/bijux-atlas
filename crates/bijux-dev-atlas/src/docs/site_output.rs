// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use serde_yaml::Value as YamlValue;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MkdocsSitePaths {
    pub docs_dir: PathBuf,
    pub site_dir: PathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SiteOutputContractConfig {
    pub minimum_file_count: usize,
    pub assets_directory: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SiteOutputStatus {
    pub docs_dir: PathBuf,
    pub site_dir: PathBuf,
    pub site_dir_exists: bool,
    pub index_exists: bool,
    pub assets_exists: bool,
    pub file_count: usize,
    pub minimum_file_count: usize,
    pub assets_directory: String,
}

fn mkdocs_yaml_path(repo_root: &Path) -> PathBuf {
    repo_root.join("mkdocs.yml")
}

fn site_output_contract_path(repo_root: &Path) -> PathBuf {
    repo_root.join("configs/docs/site-output-contract.json")
}

fn site_output_contract_schema_path(repo_root: &Path) -> PathBuf {
    repo_root.join("configs/contracts/docs-site-output.schema.json")
}

fn report_schema_path(repo_root: &Path, file_name: &str) -> PathBuf {
    repo_root.join("configs/contracts/reports").join(file_name)
}

pub fn render_stable_report_json(payload: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string_pretty(payload).map_err(|err| format!("encode failed: {err}"))
}

pub fn validate_report_schema_file(schema_path: &Path) -> Result<serde_json::Value, String> {
    let schema_text = fs::read_to_string(schema_path)
        .map_err(|err| format!("failed to read {}: {err}", schema_path.display()))?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_text)
        .map_err(|err| format!("failed to parse {}: {err}", schema_path.display()))?;
    let required = schema_json
        .get("required")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{} must declare required array", schema_path.display()))?;
    for field in ["report_id", "version", "inputs"] {
        if !required.iter().any(|value| value.as_str() == Some(field)) {
            return Err(format!("{} must require `{field}`", schema_path.display()));
        }
    }
    Ok(schema_json)
}

pub fn validate_report_value_against_schema(
    report: &serde_json::Value,
    schema_path: &Path,
) -> Result<(), String> {
    let schema_json = validate_report_schema_file(schema_path)?;
    let report_obj = report
        .as_object()
        .ok_or_else(|| "report must be a JSON object".to_string())?;
    let schema_properties = schema_json
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("{} must declare properties object", schema_path.display()))?;
    for field in schema_json
        .get("required")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        if !report_obj.contains_key(field) {
            return Err(format!("report missing required key `{field}`"));
        }
    }
    for (field, rule) in schema_properties {
        let Some(value) = report_obj.get(field) else {
            continue;
        };
        if let Some(expected) = rule.get("const") {
            if value != expected {
                return Err(format!("report field `{field}` must equal {expected}"));
            }
        }
        if let Some(type_name) = rule.get("type").and_then(serde_json::Value::as_str) {
            let type_matches = match type_name {
                "object" => value.is_object(),
                "array" => value.is_array(),
                "string" => value.is_string(),
                "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
                "number" => value.is_number(),
                "boolean" => value.is_boolean(),
                _ => true,
            };
            if !type_matches {
                return Err(format!("report field `{field}` must be {type_name}"));
            }
        }
    }
    Ok(())
}

pub fn validate_named_report(
    repo_root: &Path,
    schema_file_name: &str,
    report: &serde_json::Value,
) -> Result<(), String> {
    validate_report_value_against_schema(report, &report_schema_path(repo_root, schema_file_name))
}

pub fn closure_index_report() -> serde_json::Value {
    json!({
        "report_id": "closure-index",
        "version": 1,
        "inputs": {
            "source": "docs/_internal/governance/closure-checks.md"
        },
        "entries": [
            {
                "check_id": "REPO-003",
                "title": "Helm env subset",
                "docs_path": "docs/reference/contracts/ops/helm-env-subset.md"
            },
            {
                "check_id": "REPO-004",
                "title": "Ops profile matrix",
                "docs_path": "docs/reference/contracts/ops/profile-matrix.md"
            },
            {
                "check_id": "REPO-005",
                "title": "Docs site output",
                "docs_path": "docs/reference/contracts/docs/site-output.md"
            },
            {
                "check_id": "runtime-env-allowlist",
                "title": "Runtime env allowlist enforcement",
                "docs_path": "docs/reference/runtime/config.md"
            }
        ]
    })
}

pub fn closure_index_markdown(report: &serde_json::Value) -> Result<String, String> {
    let entries = report
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "closure index must include entries".to_string())?;
    let mut out = String::from("# Closure Index\n\n");
    out.push_str("Generated from the boundary closure map.\n\n");
    out.push_str("| Check ID | Meaning | Docs |\n|---|---|---|\n");
    for entry in entries {
        let check_id = entry
            .get("check_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "closure index entry must include check_id".to_string())?;
        let title = entry
            .get("title")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "closure index entry must include title".to_string())?;
        let docs_path = entry
            .get("docs_path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "closure index entry must include docs_path".to_string())?;
        out.push_str(&format!("| `{check_id}` | {title} | `{docs_path}` |\n"));
    }
    Ok(out)
}

pub fn report_manifest(entries: &[(&str, &str)]) -> serde_json::Value {
    let mut rows = entries
        .iter()
        .map(|(report_id, file_name)| {
            json!({
                "report_id": report_id,
                "file": file_name
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left["report_id"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["report_id"].as_str().unwrap_or_default())
    });
    json!({
        "report_id": "report-manifest",
        "version": 1,
        "inputs": {
            "generator": "bijux dev atlas docs doctor"
        },
        "reports": rows
    })
}

pub fn parse_mkdocs_site_paths(repo_root: &Path) -> Result<MkdocsSitePaths, String> {
    let path = mkdocs_yaml_path(repo_root);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let yaml: YamlValue = serde_yaml::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
    let mapping = yaml
        .as_mapping()
        .ok_or_else(|| format!("{} must be a yaml mapping", path.display()))?;

    let docs_dir = mapping
        .get(YamlValue::from("docs_dir"))
        .and_then(YamlValue::as_str)
        .unwrap_or("docs");
    let site_dir = mapping
        .get(YamlValue::from("site_dir"))
        .and_then(YamlValue::as_str)
        .unwrap_or("site");

    Ok(MkdocsSitePaths {
        docs_dir: PathBuf::from(docs_dir),
        site_dir: PathBuf::from(site_dir),
    })
}

pub fn load_site_output_contract_config(
    repo_root: &Path,
) -> Result<SiteOutputContractConfig, String> {
    let schema_path = site_output_contract_schema_path(repo_root);
    let schema_text = fs::read_to_string(&schema_path)
        .map_err(|e| format!("failed to read {}: {e}", schema_path.display()))?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_text)
        .map_err(|e| format!("failed to parse {}: {e}", schema_path.display()))?;
    if !schema_json.is_object() {
        return Err(format!(
            "{} must contain a json object schema",
            schema_path.display()
        ));
    }

    let path = site_output_contract_path(repo_root);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;

    let minimum_file_count = json
        .get("minimum_file_count")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| format!("{} must define numeric minimum_file_count", path.display()))?
        as usize;
    let assets_directory = json
        .get("assets_directory")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("{} must define string assets_directory", path.display()))?;
    if assets_directory.trim().is_empty() {
        return Err(format!(
            "{} must define a non-empty assets_directory",
            path.display()
        ));
    }

    Ok(SiteOutputContractConfig {
        minimum_file_count,
        assets_directory: assets_directory.to_string(),
    })
}

fn count_files(root: &Path) -> usize {
    let mut total = 0usize;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                total += 1;
            }
        }
    }
    total
}

pub fn collect_site_output_status(repo_root: &Path) -> Result<SiteOutputStatus, String> {
    let paths = parse_mkdocs_site_paths(repo_root)?;
    let config = load_site_output_contract_config(repo_root)?;
    let site_root = repo_root.join(&paths.site_dir);

    Ok(SiteOutputStatus {
        docs_dir: paths.docs_dir,
        site_dir: paths.site_dir,
        site_dir_exists: site_root.is_dir(),
        index_exists: site_root.join("index.html").is_file(),
        assets_exists: site_root.join(&config.assets_directory).is_dir(),
        file_count: count_files(&site_root),
        minimum_file_count: config.minimum_file_count,
        assets_directory: config.assets_directory,
    })
}

pub fn site_output_report(repo_root: &Path) -> Result<serde_json::Value, String> {
    let status = collect_site_output_status(repo_root)?;
    Ok(json!({
        "report_id": "docs-site-output",
        "version": 1,
        "inputs": {
            "mkdocs": "mkdocs.yml",
            "contract": "configs/docs/site-output-contract.json"
        },
        "docs_dir": status.docs_dir.display().to_string(),
        "site_dir": status.site_dir.display().to_string(),
        "checks": [
            {
                "id": "DOCS-SITE-001",
                "title": "mkdocs site_dir exists after build",
                "pass": status.site_dir_exists
            },
            {
                "id": "DOCS-SITE-002",
                "title": "site_dir contains index.html",
                "pass": status.index_exists
            },
            {
                "id": "DOCS-SITE-003",
                "title": "site_dir contains configured assets directory",
                "pass": status.assets_exists
            },
            {
                "id": "DOCS-SITE-FILE-COUNT",
                "title": "site_dir keeps a non-trivial file count",
                "pass": status.file_count >= status.minimum_file_count
            }
        ],
        "counts": {
            "file_count": status.file_count,
            "minimum_file_count": status.minimum_file_count
        },
        "assets_directory": status.assets_directory,
        "status": if status.site_dir_exists
            && status.index_exists
            && status.assets_exists
            && status.file_count >= status.minimum_file_count
        {
            "pass"
        } else {
            "fail"
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        root.push(format!("bijux-dev-atlas-{name}-{nanos}"));
        fs::create_dir_all(&root).expect("create temp root");
        root
    }

    fn write_contract(root: &Path) {
        let config_dir = root.join("configs/docs");
        let contracts_dir = root.join("configs/contracts");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::create_dir_all(&contracts_dir).expect("create contracts dir");
        fs::write(
            config_dir.join("site-output-contract.json"),
            serde_json::to_string_pretty(&json!({
                "minimum_file_count": 2,
                "assets_directory": "assets"
            }))
            .expect("encode contract"),
        )
        .expect("write contract");
        fs::write(
            contracts_dir.join("docs-site-output.schema.json"),
            serde_json::to_string_pretty(&json!({
                "type": "object"
            }))
            .expect("encode schema"),
        )
        .expect("write schema");
    }

    #[test]
    fn parses_declared_site_dir_and_docs_dir() {
        let root = temp_root("parse");
        fs::write(
            root.join("mkdocs.yml"),
            "docs_dir: docs-source\nsite_dir: artifacts/docs/site\n",
        )
        .expect("write mkdocs");
        let parsed = parse_mkdocs_site_paths(&root).expect("parse");
        assert_eq!(parsed.docs_dir, PathBuf::from("docs-source"));
        assert_eq!(parsed.site_dir, PathBuf::from("artifacts/docs/site"));
    }

    #[test]
    fn missing_site_dir_uses_mkdocs_default() {
        let root = temp_root("default");
        fs::write(root.join("mkdocs.yml"), "docs_dir: docs\n").expect("write mkdocs");
        let parsed = parse_mkdocs_site_paths(&root).expect("parse");
        assert_eq!(parsed.site_dir, PathBuf::from("site"));
    }

    #[test]
    fn relative_site_dir_is_preserved() {
        let root = temp_root("relative");
        fs::write(root.join("mkdocs.yml"), "site_dir: artifacts/docs/site\n")
            .expect("write mkdocs");
        let parsed = parse_mkdocs_site_paths(&root).expect("parse");
        assert_eq!(parsed.site_dir, PathBuf::from("artifacts/docs/site"));
    }

    #[test]
    fn missing_output_dir_fails_status_checks() {
        let root = temp_root("status");
        fs::write(root.join("mkdocs.yml"), "site_dir: artifacts/docs/site\n")
            .expect("write mkdocs");
        write_contract(&root);
        let status = collect_site_output_status(&root).expect("status");
        assert!(!status.site_dir_exists);
        assert!(!status.index_exists);
        assert!(!status.assets_exists);
        assert_eq!(status.file_count, 0);
    }

    #[test]
    fn validates_sample_report_against_report_schema_shape() {
        let root = temp_root("report-schema");
        let reports_dir = root.join("configs/contracts/reports");
        fs::create_dir_all(&reports_dir).expect("create reports dir");
        fs::write(
            reports_dir.join("docs-site-output.schema.json"),
            serde_json::to_string_pretty(&json!({
                "type": "object",
                "required": ["report_id", "version", "inputs"],
                "properties": {
                    "report_id": {"type": "string", "const": "docs-site-output"},
                    "version": {"type": "integer", "const": 1},
                    "inputs": {"type": "object"}
                }
            }))
            .expect("encode schema"),
        )
        .expect("write schema");
        let report = json!({
            "report_id": "docs-site-output",
            "version": 1,
            "inputs": {"mkdocs": "mkdocs.yml"}
        });
        validate_named_report(&root, "docs-site-output.schema.json", &report)
            .expect("report schema");
    }

    #[test]
    fn rejects_invalid_report_with_useful_error() {
        let root = temp_root("invalid-report");
        let reports_dir = root.join("configs/contracts/reports");
        fs::create_dir_all(&reports_dir).expect("create reports dir");
        fs::write(
            reports_dir.join("closure-summary.schema.json"),
            serde_json::to_string_pretty(&json!({
                "type": "object",
                "required": ["report_id", "version", "inputs"],
                "properties": {
                    "report_id": {"type": "string", "const": "closure-summary"},
                    "version": {"type": "integer", "const": 1},
                    "inputs": {"type": "object"}
                }
            }))
            .expect("encode schema"),
        )
        .expect("write schema");
        let report = json!({
            "report_id": "wrong-name",
            "version": 1,
            "inputs": {}
        });
        let error = validate_named_report(&root, "closure-summary.schema.json", &report)
            .expect_err("invalid");
        assert!(error.contains("report field `report_id` must equal"));
    }

    #[test]
    fn report_manifest_keeps_stable_order() {
        let manifest = report_manifest(&[("zeta", "zeta.json"), ("alpha", "alpha.json")]);
        let rows = manifest["reports"].as_array().expect("reports array");
        assert_eq!(rows[0]["report_id"], json!("alpha"));
        assert_eq!(rows[1]["report_id"], json!("zeta"));
    }
}
