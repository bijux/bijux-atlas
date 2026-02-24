// SPDX-License-Identifier: Apache-2.0

use crate::ModelVersion;
use crate::ValidationError;
use crate::{Assembly, GeneId, Release, SeqId, Species};
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
    pub dataset: crate::DatasetId,
    pub entries: Vec<ReleaseGeneIndexEntry>,
}

impl ReleaseGeneIndex {
    #[must_use]
    pub fn new(
        schema_version: String,
        dataset: crate::DatasetId,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DiffStatus {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DiffScope {
    Genes,
    Region,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct DiffRecord {
    pub gene_id: GeneId,
    pub status: DiffStatus,
    pub seqid: Option<SeqId>,
    pub start: Option<u64>,
    pub end: Option<u64>,
}

impl DiffRecord {
    #[must_use]
    pub fn new(
        gene_id: GeneId,
        status: DiffStatus,
        seqid: Option<SeqId>,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Self {
        Self {
            gene_id,
            status,
            seqid,
            start,
            end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct DiffPage {
    #[serde(default)]
    pub model_version: ModelVersion,
    pub from_release: Release,
    pub to_release: Release,
    pub species: Species,
    pub assembly: Assembly,
    pub scope: DiffScope,
    pub rows: Vec<DiffRecord>,
    pub next_cursor: Option<String>,
}

impl DiffPage {
    #[must_use]
    pub fn new(
        from_release: Release,
        to_release: Release,
        species: Species,
        assembly: Assembly,
        scope: DiffScope,
        rows: Vec<DiffRecord>,
        next_cursor: Option<String>,
    ) -> Self {
        Self {
            model_version: ModelVersion::V1,
            from_release,
            to_release,
            species,
            assembly,
            scope,
            rows,
            next_cursor,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.rows.is_empty() {
            return Err(ValidationError(
                "diff page must contain at least one row".to_string(),
            ));
        }
        Ok(())
    }
}
