// SPDX-License-Identifier: Apache-2.0
//! Canonical contract catalog built from domain-owned entrypoints.

use std::collections::BTreeSet;
use std::path::Path;

use crate::contracts::Contract;
use crate::domains;

pub struct ContractCatalogEntry {
    pub domain: &'static str,
    pub contract: Contract,
}

impl ContractCatalogEntry {
    pub fn id(&self) -> &str {
        self.contract.id.0.as_str()
    }

    pub fn title(&self) -> &str {
        self.contract.title
    }
}

#[derive(Default)]
pub struct ContractCatalog {
    entries: Vec<ContractCatalogEntry>,
}

impl ContractCatalog {
    pub fn from_entries(entries: Vec<ContractCatalogEntry>) -> Self {
        Self { entries }
    }

    pub fn load(repo_root: &Path) -> Result<Self, String> {
        let mut entries = Vec::new();
        entries.extend(load_domain_contracts("configs", domains::configs::contracts(repo_root)?)?);
        entries.extend(load_domain_contracts("docs", domains::docs::contracts(repo_root)?)?);
        entries.extend(load_domain_contracts("docker", domains::docker::contracts(repo_root)?)?);
        entries.extend(load_domain_contracts("governance", domains::governance::contracts(repo_root)?)?);
        entries.extend(load_domain_contracts("ops", domains::ops::contracts(repo_root)?)?);
        entries.sort_by(|a, b| a.domain.cmp(b.domain).then_with(|| a.id().cmp(b.id())));

        let catalog = Self::from_entries(entries);
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn entries(&self) -> &[ContractCatalogEntry] {
        &self.entries
    }

    pub fn for_domain(&self, domain: &str) -> Vec<&ContractCatalogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.domain == domain)
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        for entry in &self.entries {
            if !ids.insert(entry.id().to_string()) {
                return Err(format!("duplicate contract id `{}` in contract catalog", entry.id()));
            }
            if entry.title().trim().is_empty() {
                return Err(format!(
                    "contract `{}` in domain `{}` must declare a non-empty title",
                    entry.id(),
                    entry.domain
                ));
            }
            if entry.contract.tests.is_empty() {
                return Err(format!(
                    "contract `{}` in domain `{}` must declare at least one test case",
                    entry.id(),
                    entry.domain
                ));
            }
            for test in &entry.contract.tests {
                if test.title.trim().is_empty() {
                    return Err(format!(
                        "contract `{}` test `{}` must declare a non-empty title",
                        entry.id(),
                        test.id.0
                    ));
                }
            }
        }
        Ok(())
    }
}

fn load_domain_contracts(
    domain: &'static str,
    contracts: Vec<Contract>,
) -> Result<Vec<ContractCatalogEntry>, String> {
    let mut entries = contracts
        .into_iter()
        .map(|contract| ContractCatalogEntry { domain, contract })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| a.id().cmp(b.id()));
    if entries.is_empty() {
        return Err(format!("domain `{domain}` must register at least one contract"));
    }
    Ok(entries)
}
