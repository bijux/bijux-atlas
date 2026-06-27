// SPDX-License-Identifier: Apache-2.0

#[path = "contracts/dataset_alias_contract.rs"]
mod dataset_alias_contract;
#[path = "contracts/dataset_identity_contract.rs"]
mod dataset_identity_contract;
#[path = "contracts/dataset_key_contract.rs"]
mod dataset_key_contract;
#[path = "contracts/dataset_lifecycle_contract.rs"]
mod dataset_lifecycle_contract;
#[path = "contracts/dependency_guardrails.rs"]
mod dependency_guardrails;
#[path = "contracts/invariants_contract.rs"]
mod invariants_contract;
#[path = "contracts/model_invariants.rs"]
mod model_invariants;
#[path = "contracts/model_validation.rs"]
mod model_validation;
#[path = "contracts/non_exhaustive_guardrails.rs"]
mod non_exhaustive_guardrails;
#[path = "contracts/proptest_dataset.rs"]
mod proptest_dataset;
#[path = "contracts/proptest_region.rs"]
mod proptest_region;
#[path = "contracts/serde_contract.rs"]
mod serde_contract;
#[path = "contracts/serde_roundtrip_fixtures.rs"]
mod serde_roundtrip_fixtures;
