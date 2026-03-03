// SPDX-License-Identifier: Apache-2.0
//! Docs domain contracts and runtime adapters.

pub mod contracts;
pub mod runtime;

use crate::domains::Domain;
use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;

pub struct DocsDomain;

pub fn plugin() -> DocsDomain {
    DocsDomain
}

impl Domain for DocsDomain {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/control-plane/contracts.md", "docs/reference/reports/index.md"]
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
