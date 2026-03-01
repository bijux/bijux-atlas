// SPDX-License-Identifier: Apache-2.0
//! Contracts runner engine.
//!
//! This module provides a domain-agnostic contracts runner with deterministic ordering,
//! filterable execution, pretty and JSON output, and explicit effect gating.

pub mod configs;
pub mod crates;
pub mod control_plane;
pub mod docker;
pub mod docs;
pub mod make;
pub mod ops;
pub mod repo;
pub mod root;
pub mod runtime;

include!("engine_model.rs");
include!("engine_selection.rs");
include!("engine_runner.rs");
include!("engine_rendering.rs");
include!("engine_tests.rs");
