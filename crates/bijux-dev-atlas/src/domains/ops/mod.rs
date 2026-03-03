// SPDX-License-Identifier: Apache-2.0
//! Ops domain contracts, checks, and runtime support.

pub mod checks;
pub mod contracts;
pub mod runtime;

use crate::domains::Domain;
use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;

pub struct OpsDomain;

pub fn plugin() -> OpsDomain {
    OpsDomain
}

impl Domain for OpsDomain {
    fn name(&self) -> &'static str {
        "ops"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &[
            "docs/reference/ops-surface.md",
            "docs/reference/contracts/ops/lifecycle.md",
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
