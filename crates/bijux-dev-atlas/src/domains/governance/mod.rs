// SPDX-License-Identifier: Apache-2.0
//! Governance domain command surface.

pub mod checks;
pub mod commands;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct GovernanceDomain;

pub fn plugin() -> GovernanceDomain {
    GovernanceDomain
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for GovernanceDomain {
    fn name(&self) -> &'static str {
        "governance"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &[
            "docs/06-development/contributor-workflow.md",
            "docs/08-contracts/ownership-and-versioning.md",
        ]
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
