// SPDX-License-Identifier: Apache-2.0
//! Runnable registry backed by governance JSON registries.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::model::{
    Effect, RunnableEntry, RunnableId, RunnableKind, RunnableMode, RunnableSelection, SuiteEntry,
    SuiteId, Tag,
};

#[derive(Debug, Clone)]
pub struct RunnableRegistry {
    runnables: Vec<RunnableEntry>,
    suites: Vec<SuiteEntry>,
    index: BTreeMap<String, usize>,
}

impl RunnableRegistry {
    pub fn load(repo_root: &Path) -> Result<Self, String> {
        let suites = load_suites(repo_root)?;
        let mut runnables = Vec::new();
        runnables.extend(load_checks(repo_root)?);
        runnables.extend(load_contracts(repo_root)?);
        runnables.sort_by(|a, b| {
            a.suite
                .as_str()
                .cmp(b.suite.as_str())
                .then_with(|| a.group.cmp(&b.group))
                .then_with(|| a.id.as_str().cmp(b.id.as_str()))
        });

        let index = runnables
            .iter()
            .enumerate()
            .map(|(idx, entry)| (entry.id.as_str().to_string(), idx))
            .collect::<BTreeMap<_, _>>();

        Ok(Self {
            runnables,
            suites,
            index,
        })
    }

    pub fn all(&self) -> &[RunnableEntry] {
        &self.runnables
    }

    pub fn suites(&self) -> &[SuiteEntry] {
        &self.suites
    }

    pub fn get(&self, id: &RunnableId) -> Option<&RunnableEntry> {
        self.index.get(id.as_str()).map(|idx| &self.runnables[*idx])
    }

    pub fn select(&self, selection: &RunnableSelection) -> Vec<RunnableEntry> {
        let mut entries = self
            .runnables
            .iter()
            .filter(|entry| {
                selection
                    .suite
                    .as_ref()
                    .is_none_or(|suite| &entry.suite == suite)
            })
            .filter(|entry| {
                selection
                    .group
                    .as_ref()
                    .is_none_or(|group| &entry.group == group)
            })
            .filter(|entry| {
                selection
                    .tag
                    .as_ref()
                    .is_none_or(|tag| entry.tags.iter().any(|value| value == tag))
            })
            .filter(|entry| selection.id.as_ref().is_none_or(|id| &entry.id == id))
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| {
            a.suite
                .as_str()
                .cmp(b.suite.as_str())
                .then_with(|| a.group.cmp(&b.group))
                .then_with(|| a.id.as_str().cmp(b.id.as_str()))
        });
        entries
    }
}

#[derive(Debug, Deserialize)]
struct SuitesIndex {
    schema_version: u64,
    index_id: String,
    suites: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SuiteFile {
    schema_version: u64,
    suite_id: String,
    entries: Vec<SuiteFileEntry>,
}

#[derive(Debug, Deserialize)]
struct SuiteFileEntry {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ChecksRegistry {
    checks: Vec<CheckRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct CheckRegistryEntry {
    check_id: String,
    summary: String,
    group: String,
    commands: Vec<String>,
    report_ids: Vec<String>,
    reports: Vec<String>,
    tags: Option<Vec<String>>,
    requires_tools: Option<Vec<String>>,
    missing_tools_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContractsRegistry {
    contracts: Vec<ContractRegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct ContractRegistryEntry {
    contract_id: String,
    summary: String,
    group: String,
    runner: String,
    reports: Vec<String>,
    tags: Option<Vec<String>>,
    requires_tools: Option<Vec<String>>,
    missing_tools_policy: Option<String>,
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn load_suites(repo_root: &Path) -> Result<Vec<SuiteEntry>, String> {
    let path = repo_root.join("configs/governance/suites/suites.index.json");
    let index: SuitesIndex = load_json(&path)?;
    if index.schema_version != 1 || index.index_id != "governance-suites" {
        return Err(format!(
            "{} must declare schema_version=1 and index_id=governance-suites",
            path.display()
        ));
    }
    let mut suites = index
        .suites
        .into_iter()
        .map(|name| name.trim_end_matches(".suite.json").to_string())
        .map(|suite_name| {
            let suite_path = repo_root
                .join("configs/governance/suites")
                .join(format!("{suite_name}.suite.json"));
            let suite_file: SuiteFile = load_json(&suite_path)?;
            if suite_file.schema_version != 1 || suite_file.suite_id != suite_name {
                return Err(format!(
                    "{} must declare schema_version=1 and suite_id `{suite_name}`",
                    suite_path.display()
                ));
            }
            Ok(SuiteEntry {
                id: SuiteId::parse(&suite_file.suite_id)?,
                runnables: suite_file
                    .entries
                    .into_iter()
                    .map(|entry| RunnableId::parse(&entry.id))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    suites.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(suites)
}

fn load_checks(repo_root: &Path) -> Result<Vec<RunnableEntry>, String> {
    let path = repo_root.join("configs/governance/checks.registry.json");
    let registry: ChecksRegistry = load_json(&path)?;
    let suite = SuiteId::parse("checks")?;
    let mut entries = registry
        .checks
        .into_iter()
        .map(|entry| {
            Ok(RunnableEntry {
                id: RunnableId::parse(&entry.check_id)?,
                suite: suite.clone(),
                kind: RunnableKind::Check,
                mode: RunnableMode::Pure,
                summary: entry.summary,
                owner: "checks".to_string(),
                group: entry.group,
                tags: entry
                    .tags
                    .unwrap_or_default()
                    .iter()
                    .map(|value| Tag::parse(value))
                    .collect::<Result<Vec<_>, _>>()?,
                commands: entry.commands,
                report_ids: entry.report_ids,
                reports: entry.reports,
                required_tools: entry.requires_tools.unwrap_or_default(),
                missing_tools_policy: entry
                    .missing_tools_policy
                    .unwrap_or_else(|| "fail".to_string()),
                effects_required: vec![Effect::Subprocess, Effect::FsWrite],
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(entries)
}

fn load_contracts(repo_root: &Path) -> Result<Vec<RunnableEntry>, String> {
    let path = repo_root.join("configs/governance/contracts.registry.json");
    let registry: ContractsRegistry = load_json(&path)?;
    let suite = SuiteId::parse("contracts")?;
    let mut entries = registry
        .contracts
        .into_iter()
        .map(|entry| {
            let mode = if entry.runner.contains("--mode effect") {
                RunnableMode::Effect
            } else {
                RunnableMode::Pure
            };
            let effects_required = if mode == RunnableMode::Effect {
                vec![Effect::Subprocess, Effect::FsWrite]
            } else {
                vec![Effect::FsWrite]
            };
            Ok(RunnableEntry {
                id: RunnableId::parse(&entry.contract_id)?,
                suite: suite.clone(),
                kind: RunnableKind::Contract,
                mode,
                summary: entry.summary,
                owner: "contracts".to_string(),
                group: entry.group,
                tags: entry
                    .tags
                    .unwrap_or_default()
                    .iter()
                    .map(|value| Tag::parse(value))
                    .collect::<Result<Vec<_>, _>>()?,
                commands: vec![entry.runner],
                report_ids: entry.reports.clone(),
                reports: entry.reports,
                required_tools: entry.requires_tools.unwrap_or_default(),
                missing_tools_policy: entry
                    .missing_tools_policy
                    .unwrap_or_else(|| "fail".to_string()),
                effects_required,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(entries)
}
