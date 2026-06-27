// SPDX-License-Identifier: Apache-2.0

#[path = "model_contracts/dataset_alias_contract.rs"]
mod dataset_alias_contract;
#[path = "model_contracts/dataset_identity_contract.rs"]
mod dataset_identity_contract;
#[path = "model_contracts/dataset_key_contract.rs"]
mod dataset_key_contract;
#[path = "model_contracts/dataset_lifecycle_contract.rs"]
mod dataset_lifecycle_contract;
#[path = "model_contracts/dependency_guardrails.rs"]
mod dependency_guardrails;
#[path = "model_contracts/invariants_contract.rs"]
mod invariants_contract;
#[path = "model_contracts/model_invariants.rs"]
mod model_invariants;
#[path = "model_contracts/model_validation.rs"]
mod model_validation;
#[path = "model_contracts/non_exhaustive_guardrails.rs"]
mod non_exhaustive_guardrails;
#[path = "model_contracts/proptest_dataset.rs"]
mod proptest_dataset;
#[path = "model_contracts/proptest_region.rs"]
mod proptest_region;
#[path = "model_contracts/serde_contract.rs"]
mod serde_contract;
#[path = "model_contracts/serde_roundtrip_fixtures.rs"]
mod serde_roundtrip_fixtures;
