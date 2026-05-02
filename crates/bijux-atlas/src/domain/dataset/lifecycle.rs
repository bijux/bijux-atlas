// SPDX-License-Identifier: Apache-2.0

use super::{DatasetId, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DatasetLifecycleState {
    Draft,
    Published,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct DatasetLifecycleTransition {
    pub schema_version: u64,
    pub dataset: DatasetId,
    pub from_state: DatasetLifecycleState,
    pub to_state: DatasetLifecycleState,
    pub transition_at: String,
    pub transition_by: String,
    pub reason: String,
    pub manifest_sha256: String,
    pub sqlite_sha256: String,
}

impl DatasetLifecycleTransition {
    #[must_use]
    pub fn publish(
        dataset: DatasetId,
        transition_at: String,
        transition_by: String,
        reason: String,
        manifest_sha256: String,
        sqlite_sha256: String,
    ) -> Self {
        Self {
            schema_version: 1,
            dataset,
            from_state: DatasetLifecycleState::Draft,
            to_state: DatasetLifecycleState::Published,
            transition_at,
            transition_by,
            reason,
            manifest_sha256,
            sqlite_sha256,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != 1 {
            return Err(ValidationError(
                "dataset lifecycle transition schema_version must be 1".to_string(),
            ));
        }
        if !matches!(
            (self.from_state, self.to_state),
            (DatasetLifecycleState::Draft, DatasetLifecycleState::Published)
        ) {
            return Err(ValidationError(
                "dataset lifecycle transition must be draft -> published".to_string(),
            ));
        }
        if self.transition_at.trim().is_empty() {
            return Err(ValidationError(
                "dataset lifecycle transition_at must not be empty".to_string(),
            ));
        }
        if self.transition_by.trim().is_empty() {
            return Err(ValidationError(
                "dataset lifecycle transition_by must not be empty".to_string(),
            ));
        }
        if self.reason.trim().is_empty() {
            return Err(ValidationError(
                "dataset lifecycle reason must not be empty".to_string(),
            ));
        }
        for (field, value) in [
            ("manifest_sha256", self.manifest_sha256.as_str()),
            ("sqlite_sha256", self.sqlite_sha256.as_str()),
        ] {
            if !is_sha256_hex(value) {
                return Err(ValidationError(format!(
                    "dataset lifecycle field {field} must be 64-char lowercase sha256 hex"
                )));
            }
        }
        Ok(())
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
