// SPDX-License-Identifier: Apache-2.0
//! Registry loading, validation, and indexing entrypoints.
//!
//! This module consolidates registry-oriented logic that had been scattered between engine and
//! configs contracts, while preserving existing callers through re-exports.

pub mod configs;
pub mod reports;
mod runnable;

pub use crate::core::{
    expand_suite, explain_output, list_output, load_registry, registry_doctor, select_checks,
    validate_registry, Registry, RegistryDoctorReport, SuiteSpec, DEFAULT_REGISTRY_PATH,
};
pub use reports::{ReportRegistry, ReportRegistryEntry, REPORTS_REGISTRY_PATH};
pub use runnable::RunnableRegistry;
