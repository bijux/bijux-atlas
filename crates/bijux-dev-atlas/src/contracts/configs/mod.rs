// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{Contract, ContractId, RunContext, TestCase, TestId, TestKind, TestResult, Violation};

include!("configs_registry_support.rs");
include!("configs_core_contracts.rs");
include!("configs_registry_contracts.rs");
include!("configs_authority_contracts.rs");
include!("configs_schema_contracts.rs");
include!("configs_surface_contracts.rs");
