// SPDX-License-Identifier: Apache-2.0

#[path = "server/async_runtime_contract.rs"]
mod async_runtime_contract;
#[path = "server/download_then_serve.rs"]
mod download_then_serve;
#[path = "server/import_boundary_guardrails.rs"]
mod import_boundary_guardrails;
#[path = "server/logging_contracts.rs"]
mod logging_contracts;
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
