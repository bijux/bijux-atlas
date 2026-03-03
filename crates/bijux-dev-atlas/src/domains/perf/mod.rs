// SPDX-License-Identifier: Apache-2.0
//! Performance domain registration.

pub mod runtime;

use crate::domains::Domain;
use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;

pub struct PerfDomain;

pub fn plugin() -> PerfDomain {
    PerfDomain
}

impl Domain for PerfDomain {
    fn name(&self) -> &'static str {
        "perf"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/control-plane/performance-budget.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["cargo", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, _registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        Vec::new()
    }
}
