// SPDX-License-Identifier: Apache-2.0
//! Release domain registration.

use crate::domains::Domain;
use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;

pub struct ReleaseDomain;

pub fn plugin() -> ReleaseDomain {
    ReleaseDomain
}

impl Domain for ReleaseDomain {
    fn name(&self) -> &'static str {
        "release"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/product/release-model.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["cargo", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, _registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        Vec::new()
    }
}
