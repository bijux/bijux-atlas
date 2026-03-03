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
pub mod make;
pub mod ops;
pub mod repo;
pub mod root;
pub mod runtime;
