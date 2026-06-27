// SPDX-License-Identifier: Apache-2.0

use crate::dataset::{DatasetId, ModelVersion, ValidationError};
use crate::{GeneId, SeqId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GeneSignatureInput {
    pub gene_id: GeneId,
    pub name: String,
    pub biotype: String,
    pub seqid: SeqId,
    pub start: u64,
    pub end: u64,
    pub transcript_count: u64,
}

impl GeneSignatureInput {
    #[must_use]
    pub fn new(
        gene_id: GeneId,
        name: String,
        biotype: String,
        seqid: SeqId,
        start: u64,
        end: u64,
        transcript_count: u64,
    ) -> Self {
        Self {
            gene_id,
            name,
            biotype,
            seqid,
            start,
            end,
            transcript_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ReleaseGeneIndexEntry {
    pub gene_id: GeneId,
    pub seqid: SeqId,
    pub start: u64,
    pub end: u64,
    pub signature_sha256: String,
}

impl ReleaseGeneIndexEntry {
    #[must_use]
    pub fn new(
        gene_id: GeneId,
        seqid: SeqId,
        start: u64,
        end: u64,
        signature_sha256: String,
    ) -> Self {
        Self {
            gene_id,
            seqid,
            start,
            end,
            signature_sha256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct ReleaseGeneIndex {
    #[serde(default)]
    pub model_version: ModelVersion,
    pub schema_version: String,
    pub dataset: DatasetId,
    pub entries: Vec<ReleaseGeneIndexEntry>,
}

impl ReleaseGeneIndex {
    #[must_use]
    pub fn new(
        schema_version: String,
        dataset: DatasetId,
        entries: Vec<ReleaseGeneIndexEntry>,
    ) -> Self {
        Self {
            model_version: ModelVersion::V1,
            schema_version,
            dataset,
            entries,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version.trim().is_empty() {
            return Err(ValidationError(
                "release gene index schema_version must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}
