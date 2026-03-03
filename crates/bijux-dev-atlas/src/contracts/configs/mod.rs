// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

include!("configs_registry_model.rs");
include!("configs_registry_indexing.rs");
include!("configs_core_contracts.rs");
include!("configs_registry_contracts.rs");
include!("configs_authority_contracts.rs");
include!("configs_schema_contracts.rs");
include!("configs_surface_contracts.rs");

pub fn generated_index_payload(repo_root: &Path) -> Result<serde_json::Value, String> {
    generated_index_json(repo_root)
}
