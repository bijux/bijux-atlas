// SPDX-License-Identifier: Apache-2.0
//! Release domain registration.

pub mod commands;
pub mod runtime;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct ReleaseDomain;

pub fn plugin() -> ReleaseDomain {
    ReleaseDomain
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for ReleaseDomain {
    fn name(&self) -> &'static str {
        "release"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/bijux-atlas-dev/delivery/release-and-versioning.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["cargo", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, _registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        Vec::new()
    }
}
