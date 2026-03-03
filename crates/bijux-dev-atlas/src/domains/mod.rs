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

pub use loader::{
    load_domains, Domain, DomainCatalog, DomainEvent, DomainRegistration, ToolingContract,
};
