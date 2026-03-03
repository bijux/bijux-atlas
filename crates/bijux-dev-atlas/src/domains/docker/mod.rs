// SPDX-License-Identifier: Apache-2.0
//! Docker domain contracts canonical surface.

pub mod contracts;

use crate::contracts::Contract;
use crate::domains::Domain;
use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;
use std::path::Path;

pub struct DockerDomain;

pub fn plugin() -> DockerDomain {
    DockerDomain
}

pub fn contracts(repo_root: &Path) -> Result<Vec<Contract>, String> {
    crate::contracts::docker::contracts(repo_root)
}

impl Domain for DockerDomain {
    fn name(&self) -> &'static str {
        "docker"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["docs/control-plane/tooling-dependencies.md", "docs/reference/contracts/security.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["docker", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        registry
            .all()
            .iter()
            .filter(|entry| entry.group.contains("docker") || entry.id.as_str().contains("DOCKER"))
            .cloned()
            .collect()
    }
}
