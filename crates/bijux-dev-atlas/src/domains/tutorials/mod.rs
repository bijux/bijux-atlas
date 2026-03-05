// SPDX-License-Identifier: Apache-2.0
//! Tutorials domain registration.

pub mod commands;
pub mod checks;
pub mod contracts;
pub mod runtime;

use crate::contracts::Contract;
use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;
use std::path::Path;

pub struct TutorialsDomain;

pub fn plugin() -> TutorialsDomain {
    TutorialsDomain
}

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    contracts::contracts(repo_root)
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for TutorialsDomain {
    fn name(&self) -> &'static str {
        "tutorials"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &[
            "docs/tutorials/index.md",
            "docs/tutorials/run-with-dev-atlas.md",
        ]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        registry
            .all()
            .iter()
            .filter(|entry| entry.group.contains("tutorial") || entry.id.as_str().contains("TUTORIAL"))
            .cloned()
            .collect()
    }
}
