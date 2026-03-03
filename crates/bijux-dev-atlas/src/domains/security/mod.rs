// SPDX-License-Identifier: Apache-2.0
//! Security domain registration.

use crate::domains::Domain;
use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;

pub struct SecurityDomain;

pub fn plugin() -> SecurityDomain {
    SecurityDomain
}

impl Domain for SecurityDomain {
    fn name(&self) -> &'static str {
        "security"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/operations/supply-chain-policies.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["cargo", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, _registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        _registry
            .all()
            .iter()
            .filter(|entry| {
                entry.group.contains("security") || entry.id.as_str().contains("SECURITY")
            })
            .cloned()
            .collect()
    }
}
