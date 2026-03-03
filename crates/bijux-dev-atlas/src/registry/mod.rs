// SPDX-License-Identifier: Apache-2.0
//! Registry loading, validation, and indexing entrypoints.
//!
//! This module consolidates registry-oriented logic that had been scattered between engine and
//! configs contracts, while preserving existing callers through re-exports.

pub mod configs;
pub mod checks;
pub mod contracts;
pub mod contract_modes;
pub mod reports;
pub mod routes;
mod runnable;

pub use crate::core::{
    expand_suite, explain_output, list_output, load_registry, registry_doctor, select_checks,
    validate_registry, Registry, RegistryDoctorReport, SuiteSpec, DEFAULT_REGISTRY_PATH,
};
pub use checks::{CheckCatalog, CheckCatalogEntry};
pub use contracts::{ContractCatalog, ContractCatalogEntry};
pub use contract_modes::{
    ContractMode, ContractModesFile, ContractModesValidation, CONTRACT_MODES_PATH,
    CONTRACT_MODES_SCHEMA_PATH,
};
pub use reports::{
    ReportArtifactValidation, ReportCatalogValidation, ReportProgress, ReportProgressRow,
    ReportRegistry, ReportRegistryEntry, REPORTS_REGISTRY_PATH, REPORTS_REGISTRY_SCHEMA_PATH,
};
pub use routes::{command_routes, validate_command_routes};
pub use runnable::RunnableRegistry;
