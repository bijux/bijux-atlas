// SPDX-License-Identifier: Apache-2.0
//! Canonical domain plugin surfaces for the control plane.

mod loader;

pub mod configs;
pub mod docker;
pub mod docs;
pub mod governance;
pub mod ops;
pub mod perf;
pub mod release;
pub mod security;
pub mod tutorials;

pub const ALL_DOMAIN_NAMES: &[&str] = &[
    "configs",
    "docs",
    "docker",
    "governance",
    "ops",
    "perf",
    "release",
    "security",
    "tutorials",
];

pub fn all_domains() -> &'static [&'static str] {
    ALL_DOMAIN_NAMES
}

pub use loader::{
    load_domains, Domain, DomainCatalog, DomainEvent, DomainRegistration, ToolingContract,
};
