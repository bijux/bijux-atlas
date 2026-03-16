// SPDX-License-Identifier: Apache-2.0
//! Docs domain runtime adapters.

pub mod commands;
pub mod runtime;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct DocsDomain;

pub fn plugin() -> DocsDomain {
    DocsDomain
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for DocsDomain {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &[
            "docs/08-contracts/automation-contracts.md",
            "docs/07-reference/automation-reports-reference.md",
        ]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        registry
            .all()
            .iter()
            .filter(|entry| entry.group.contains("docs") || entry.id.as_str().contains("DOC"))
            .cloned()
            .collect()
    }
}
