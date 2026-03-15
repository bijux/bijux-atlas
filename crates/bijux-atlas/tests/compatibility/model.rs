// SPDX-License-Identifier: Apache-2.0

#[path = "model/dataset_key_contract.rs"]
mod dataset_key_contract;
#[path = "model/dependency_guardrails.rs"]
mod dependency_guardrails;
#[path = "model/model_invariants.rs"]
mod model_invariants;
#[path = "model/model_validation.rs"]
mod model_validation;
#[path = "model/non_exhaustive_guardrails.rs"]
mod non_exhaustive_guardrails;
#[path = "model/proptest_dataset.rs"]
mod proptest_dataset;
#[path = "model/proptest_region.rs"]
mod proptest_region;
#[path = "model/serde_contract.rs"]
mod serde_contract;
#[path = "model/serde_roundtrip_fixtures.rs"]
mod serde_roundtrip_fixtures;
