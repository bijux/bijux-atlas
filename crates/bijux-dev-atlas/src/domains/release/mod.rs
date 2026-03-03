// SPDX-License-Identifier: Apache-2.0
//! Release domain registration.

pub mod runtime;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct ReleaseDomain;

pub fn plugin() -> ReleaseDomain {
    ReleaseDomain
}

pub fn routes() -> Vec<CommandRoute> {
    vec![CommandRoute::new(
        "release",
        "release",
        "release",
        "Run release verification commands",
    )]
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
