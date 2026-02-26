// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{ArtifactPath, CheckId, Severity, Violation, ViolationId};
use serde_yaml::Value as YamlValue;

use crate::core::{CheckContext, CheckError, CheckFn};

const OPS_TEXT_EXTENSIONS: [&str; 5] = ["md", "json", "toml", "yaml", "yml"];
mod documentation_and_config_checks;
mod governance_checks;
mod governance_repo_checks;
mod surface_contract_checks;
use documentation_and_config_checks::*;
use governance_checks::*;
use governance_repo_checks::*;
use surface_contract_checks::*;

include!("ops/ops/check_registry_dispatch.rs");
include!("ops/ops/shared_helpers_and_surface_contracts.rs");
include!("ops/ops/schema_and_manifest_checks.rs");
include!("ops/ops/inventory_and_artifact_checks.rs");
