// SPDX-License-Identifier: Apache-2.0
//! Governance domain contracts canonical surface.

pub mod contracts;

use crate::contracts::Contract;
use crate::domains::Domain;
use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;
use std::path::Path;

pub struct GovernanceDomain;

pub fn plugin() -> GovernanceDomain {
    GovernanceDomain
}

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    let mut rows = crate::contracts::repo::contracts(repo_root)?;
    rows.extend(crate::contracts::root::contracts(repo_root)?);
    rows.extend(crate::contracts::runtime::contracts(repo_root)?);
    Ok(rows)
}

impl Domain for GovernanceDomain {
    fn name(&self) -> &'static str {
        "governance"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/_internal/governance/checks-and-contracts.md", "docs/_internal/architecture/layers.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["git", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        registry
            .all()
            .iter()
            .filter(|entry| {
                entry.group.contains("repo")
                    || entry.group.contains("root")
                    || entry.id.as_str().contains("ROOT")
                    || entry.id.as_str().contains("REPO")
            })
            .cloned()
            .collect()
    }
}
