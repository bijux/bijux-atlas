// SPDX-License-Identifier: Apache-2.0
//! Contracts runner engine.
//!
//! This module provides a domain-agnostic contracts runner with deterministic ordering,
//! filterable execution, pretty and JSON output, and explicit effect gating.

pub mod docker;
pub mod make;
pub mod ops;

include!("engine_model.inc.rs");
include!("engine_selection.inc.rs");
include!("engine_runner.inc.rs");
include!("engine_rendering.inc.rs");
include!("engine_tests.inc.rs");
