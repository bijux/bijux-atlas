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
}
