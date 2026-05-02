// SPDX-License-Identifier: Apache-2.0

use super::{DatasetId, ValidationError};
use crate::domain::canonical;
use crate::domain::sha256_hex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct DatasetIdentity {
    pub release_id: String,
    pub source_fingerprint_sha256: String,
    pub build_fingerprint_sha256: String,
    pub artifact_fingerprint_sha256: String,
    pub canonical_metadata_sha256: String,
}

impl DatasetIdentity {
    #[must_use]
    pub fn new(
        release_id: String,
        source_fingerprint_sha256: String,
        build_fingerprint_sha256: String,
        artifact_fingerprint_sha256: String,
        canonical_metadata_sha256: String,
    ) -> Self {
        Self {
            release_id,
            source_fingerprint_sha256,
            build_fingerprint_sha256,
            artifact_fingerprint_sha256,
            canonical_metadata_sha256,
        }
    }

    pub fn from_components(
        dataset: &DatasetId,
        source_component: &serde_json::Value,
        build_component: &serde_json::Value,
        artifact_component: &serde_json::Value,
    ) -> Result<Self, ValidationError> {
        let source_bytes = canonical::stable_json_bytes(source_component)
            .map_err(|err| ValidationError(err.to_string()))?;
        let build_bytes = canonical::stable_json_bytes(build_component)
            .map_err(|err| ValidationError(err.to_string()))?;
        let artifact_bytes = canonical::stable_json_bytes(artifact_component)
            .map_err(|err| ValidationError(err.to_string()))?;

        let source_fingerprint_sha256 = sha256_hex(&source_bytes);
        let build_fingerprint_sha256 = sha256_hex(&build_bytes);
        let artifact_fingerprint_sha256 = sha256_hex(&artifact_bytes);

        let release_id = dataset.canonical_string();
        let canonical_metadata_sha256 = canonical_identity_hash(
            &release_id,
            &source_fingerprint_sha256,
            &build_fingerprint_sha256,
            &artifact_fingerprint_sha256,
        )?;

        Ok(Self::new(
            release_id,
            source_fingerprint_sha256,
            build_fingerprint_sha256,
            artifact_fingerprint_sha256,
            canonical_metadata_sha256,
        ))
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.release_id.trim().is_empty() {
            return Err(ValidationError(
                "identity release_id must not be empty".to_string(),
            ));
        }
        for (name, value) in [
            ("source_fingerprint_sha256", &self.source_fingerprint_sha256),
            ("build_fingerprint_sha256", &self.build_fingerprint_sha256),
            (
                "artifact_fingerprint_sha256",
                &self.artifact_fingerprint_sha256,
            ),
            ("canonical_metadata_sha256", &self.canonical_metadata_sha256),
        ] {
            if !is_sha256_hex(value) {
                return Err(ValidationError(format!(
                    "identity field {name} must be 64-char lowercase sha256 hex"
                )));
            }
        }

        let expected = canonical_identity_hash(
            &self.release_id,
            &self.source_fingerprint_sha256,
            &self.build_fingerprint_sha256,
            &self.artifact_fingerprint_sha256,
        )?;
        if expected != self.canonical_metadata_sha256 {
            return Err(ValidationError(
                "identity canonical_metadata_sha256 does not match canonical serialized identity"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

pub fn canonical_identity_hash(
    release_id: &str,
    source_fingerprint_sha256: &str,
    build_fingerprint_sha256: &str,
    artifact_fingerprint_sha256: &str,
) -> Result<String, ValidationError> {
    let payload = serde_json::json!({
        "release_id": release_id,
        "source_fingerprint_sha256": source_fingerprint_sha256,
        "build_fingerprint_sha256": build_fingerprint_sha256,
        "artifact_fingerprint_sha256": artifact_fingerprint_sha256
    });
    let bytes =
        canonical::stable_json_bytes(&payload).map_err(|err| ValidationError(err.to_string()))?;
    Ok(sha256_hex(&bytes))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
