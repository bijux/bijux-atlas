// SPDX-License-Identifier: Apache-2.0
//! Ops domain checks and runtime support.

pub mod checks;
pub mod commands;
pub mod runtime;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct OpsDomain;

pub fn plugin() -> OpsDomain {
    OpsDomain
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for OpsDomain {
    fn name(&self) -> &'static str {
        "ops"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &[
            "docs/bijux-atlas-dev/automation/automation-command-surface.md",
            "docs/bijux-atlas/contracts/operational-contracts.md",
        ]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["helm", "kubeconform", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        registry
            .all()
            .iter()
            .filter(|entry| entry.group.contains("ops") || entry.id.as_str().contains("OPS"))
            .cloned()
            .collect()
    }
}
