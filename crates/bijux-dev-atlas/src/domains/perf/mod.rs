// SPDX-License-Identifier: Apache-2.0
//! Performance domain registration.

pub mod commands;
pub mod runtime;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct PerfDomain;

pub fn plugin() -> PerfDomain {
    PerfDomain
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for PerfDomain {
    fn name(&self) -> &'static str {
        "perf"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/06-development/automation-control-plane.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["cargo", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, _registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        Vec::new()
    }
}
