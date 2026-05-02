// SPDX-License-Identifier: Apache-2.0

use super::{DatasetId, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct LatestAliasRecord {
    pub schema_version: u64,
    pub alias: String,
    pub dataset: DatasetId,
    pub policy: String,
    pub updated_at: String,
    pub updated_by: String,
    pub catalog_sha256: String,
}

impl LatestAliasRecord {
    #[must_use]
    pub fn new(
        dataset: DatasetId,
        policy: String,
        updated_at: String,
        updated_by: String,
        catalog_sha256: String,
    ) -> Self {
        Self {
            schema_version: 1,
            alias: "latest".to_string(),
            dataset,
            policy,
            updated_at,
            updated_by,
            catalog_sha256,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != 1 {
            return Err(ValidationError(
                "latest alias schema_version must be 1".to_string(),
            ));
        }
        if self.alias != "latest" {
            return Err(ValidationError(
                "latest alias record alias must be exactly \"latest\"".to_string(),
            ));
        }
        if self.policy.trim().is_empty() {
            return Err(ValidationError(
                "latest alias policy must not be empty".to_string(),
            ));
        }
        if self.updated_at.trim().is_empty() {
            return Err(ValidationError(
                "latest alias updated_at must not be empty".to_string(),
            ));
        }
        if self.updated_by.trim().is_empty() {
            return Err(ValidationError(
                "latest alias updated_by must not be empty".to_string(),
            ));
        }
        if self.catalog_sha256.trim().is_empty() {
            return Err(ValidationError(
                "latest alias catalog_sha256 must not be empty".to_string(),
            ));
        }
        if !is_sha256_hex(&self.catalog_sha256) {
            return Err(ValidationError(
                "latest alias catalog_sha256 must be 64-char lowercase sha256 hex".to_string(),
            ));
        }
        Ok(())
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
