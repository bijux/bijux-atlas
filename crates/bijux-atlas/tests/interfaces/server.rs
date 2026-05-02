// SPDX-License-Identifier: Apache-2.0

#[path = "server/api-contracts.rs"]
mod api_contracts;
#[path = "server/api_contracts_support/mod.rs"]
mod api_contracts_support;
#[path = "server/api_surface_snapshots.rs"]
mod api_surface_snapshots;
#[path = "server/async_runtime_contract.rs"]
mod async_runtime_contract;
#[path = "server/download_then_serve.rs"]
mod download_then_serve;
#[path = "server/endpoints_contract.rs"]
mod endpoints_contract;
#[path = "server/core_route_contracts.rs"]
mod core_route_contracts;
#[path = "server/import_boundary_guardrails.rs"]
mod import_boundary_guardrails;
#[path = "server/key_endpoints_golden.rs"]
mod key_endpoints_golden;
#[path = "server/logging_contracts.rs"]
mod logging_contracts;
#[path = "server/observability_contract.rs"]
mod observability_contract;
#[path = "server/p99-regression.rs"]
mod p99_regression;
#[path = "server/redis_optional.rs"]
mod redis_optional;
#[path = "server/runtime_env_contract_startup.rs"]
mod runtime_env_contract_startup;
#[path = "server/s3_backend.rs"]
mod s3_backend;
#[path = "server/schema_evolution_regression.rs"]
mod schema_evolution_regression;
#[path = "server/security_input_resilience.rs"]
mod security_input_resilience;
