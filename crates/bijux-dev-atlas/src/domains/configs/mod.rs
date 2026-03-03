// SPDX-License-Identifier: Apache-2.0
//! Configs domain contracts and registry-facing plugin surface.

pub mod commands;
pub mod contracts;

use crate::contracts::Contract;
use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;
use std::path::Path;

pub struct ConfigsDomain;

pub fn plugin() -> ConfigsDomain {
    ConfigsDomain
}

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    crate::contracts::configs::contracts(repo_root)
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for ConfigsDomain {
    fn name(&self) -> &'static str {
        "configs"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/reference/configs.md", "docs/reference/registry/index.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        registry
            .all()
            .iter()
            .filter(|entry| entry.group.contains("configs") || entry.id.as_str().contains("CONFIGS"))
            .cloned()
            .collect()
    }
}
