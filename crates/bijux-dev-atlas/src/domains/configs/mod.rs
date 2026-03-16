// SPDX-License-Identifier: Apache-2.0
//! Configs domain registry-facing plugin surface.

pub mod commands;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct ConfigsDomain;

pub fn plugin() -> ConfigsDomain {
    ConfigsDomain
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for ConfigsDomain {
    fn name(&self) -> &'static str {
        "configs"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &[
            "docs/07-reference/runtime-config-reference.md",
            "docs/06-development/workspace-and-tooling.md",
        ]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        registry
            .all()
            .iter()
            .filter(|entry| {
                entry.group.contains("configs") || entry.id.as_str().contains("CONFIGS")
            })
            .cloned()
            .collect()
    }
}
