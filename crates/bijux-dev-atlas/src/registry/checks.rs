// SPDX-License-Identifier: Apache-2.0
//! Canonical check catalog backed by the governance checks registry.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::model::DocRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckCatalogEntry {
    pub domain: &'static str,
    pub check_id: String,
    pub summary: String,
    pub group: String,
    pub tags: Vec<String>,
    pub commands: Vec<String>,
    pub doc_ref: DocRef,
}

#[derive(Debug, Default)]
pub struct CheckCatalog {
    entries: Vec<CheckCatalogEntry>,
}

impl CheckCatalog {
    pub fn from_entries(entries: Vec<CheckCatalogEntry>) -> Self {
        Self { entries }
    }

    pub fn load(repo_root: &Path) -> Result<Self, String> {
        let path = repo_root.join("configs/governance/checks.registry.json");
        let text =
            fs::read_to_string(&path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
        let registry: ChecksRegistry =
            serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))?;
        if registry.schema_version != 1 || registry.registry_id != "checks-registry" {
            return Err(format!(
                "{} must declare schema_version=1 and registry_id=checks-registry",
                path.display()
            ));
        }

        let mut entries = registry
            .checks
            .into_iter()
            .map(|row| {
                let domain = infer_domain(&row.group, &row.check_id);
                Ok(CheckCatalogEntry {
                    domain,
                    check_id: row.check_id,
                    summary: row.summary,
                    group: row.group,
                    tags: row.tags.unwrap_or_default(),
                    commands: row.commands,
                    doc_ref: default_doc_ref(domain),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        entries.sort_by(|a, b| a.domain.cmp(b.domain).then_with(|| a.check_id.cmp(&b.check_id)));

        let catalog = Self::from_entries(entries);
        catalog.validate(repo_root)?;
        Ok(catalog)
    }

    pub fn entries(&self) -> &[CheckCatalogEntry] {
        &self.entries
    }

    pub fn for_domain(&self, domain: &str) -> Vec<&CheckCatalogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.domain == domain)
            .collect()
    }

    pub fn validate(&self, repo_root: &Path) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        for entry in &self.entries {
            if entry.check_id.trim().is_empty() {
                return Err(format!("domain `{}` contains an empty check id", entry.domain));
            }
            if !ids.insert(entry.check_id.clone()) {
                return Err(format!("duplicate check id `{}` in check catalog", entry.check_id));
            }
            if entry.summary.trim().is_empty() {
                return Err(format!(
                    "check `{}` in domain `{}` must declare a non-empty summary",
                    entry.check_id, entry.domain
                ));
            }
            if entry.tags.is_empty() {
                return Err(format!(
                    "check `{}` in domain `{}` must declare at least one tag",
                    entry.check_id, entry.domain
                ));
            }
            if entry.doc_ref.title.trim().is_empty() {
                return Err(format!(
                    "check `{}` in domain `{}` must declare a non-empty doc title",
                    entry.check_id, entry.domain
                ));
            }
            let doc_path = repo_root.join(entry.doc_ref.path);
            if !doc_path.is_file() {
                return Err(format!(
                    "check `{}` in domain `{}` references missing docs file `{}`",
                    entry.check_id, entry.domain, entry.doc_ref.path
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct ChecksRegistry {
    schema_version: u64,
    registry_id: String,
    checks: Vec<CheckRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct CheckRegistryEntry {
    check_id: String,
    summary: String,
    group: String,
    commands: Vec<String>,
    tags: Option<Vec<String>>,
}

fn infer_domain(group: &str, check_id: &str) -> &'static str {
    if group.contains("docs") || check_id.contains("DOCS") {
        "docs"
    } else if group.contains("configs") || check_id.contains("CONFIGS") {
        "configs"
    } else if group.contains("docker") || check_id.contains("DOCKER") {
        "docker"
    } else if group.contains("ops") || check_id.contains("OPS") {
        "ops"
    } else {
        "governance"
    }
}

fn default_doc_ref(domain: &'static str) -> DocRef {
    match domain {
        "configs" => DocRef::new("docs/reference/configs.md", None, "Configs Reference"),
        "docs" => DocRef::new("docs/_internal/governance/checks/docs-checks.md", None, "Docs Checks"),
        "docker" => DocRef::new("docs/reference/contracts/index.md", None, "Contract Reference"),
        "ops" => DocRef::new("docs/reference/ops-surface.md", None, "Ops Surface"),
        _ => DocRef::new(
            "docs/_internal/governance/checks-and-contracts.md",
            None,
            "Checks And Contracts",
        ),
    }
}

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}
