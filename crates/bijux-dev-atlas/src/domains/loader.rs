// SPDX-License-Identifier: Apache-2.0
//! Domain registration and runtime loading.

use crate::model::RunnableEntry;
use crate::registry::RunnableRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolingContract {
    pub required_tools: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainEvent {
    pub domain: &'static str,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DomainRegistration {
    pub name: &'static str,
    pub docs_links: Vec<&'static str>,
    pub tooling: ToolingContract,
}

#[derive(Debug, Clone)]
pub struct DomainCatalog {
    pub registration: DomainRegistration,
    pub runnables: Vec<RunnableEntry>,
}

pub trait Domain {
    fn name(&self) -> &'static str;
    fn docs_links(&self) -> &'static [&'static str];
    fn required_tools(&self) -> &'static [&'static str];
    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry>;
}

pub fn load_domains(repo_root: &std::path::Path) -> Result<Vec<DomainCatalog>, String> {
    let registry = RunnableRegistry::load(repo_root)?;
    let domains: Vec<Box<dyn Domain>> = vec![
        Box::new(crate::domains::configs::plugin()),
        Box::new(crate::domains::docs::plugin()),
        Box::new(crate::domains::ops::plugin()),
        Box::new(crate::domains::governance::plugin()),
        Box::new(crate::domains::security::plugin()),
    ];
    Ok(domains
        .into_iter()
        .map(|domain| DomainCatalog {
            registration: DomainRegistration {
                name: domain.name(),
                docs_links: domain.docs_links().to_vec(),
                tooling: ToolingContract {
                    required_tools: domain.required_tools().to_vec(),
                },
            },
            runnables: domain.load_runnables(&registry),
        })
        .collect())
}
