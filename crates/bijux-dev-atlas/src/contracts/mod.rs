// SPDX-License-Identifier: Apache-2.0
//! Contracts runner engine.
//!
//! This module provides a domain-agnostic contracts runner with deterministic ordering,
//! filterable execution, pretty and JSON output, and explicit effect gating.

pub use crate::engine::*;
pub use crate::model::engine::*;

pub mod configs;
pub mod control_plane;
pub mod crates;
pub mod docker;
pub mod docs;
pub mod drift;
pub mod governance_enforcement;
pub mod make;
pub mod metrics_registry;
pub mod ops;
pub mod repo;
pub mod reproducibility;
pub mod root;
pub mod runtime;
pub mod system_invariants;
pub mod tracing_registry;
