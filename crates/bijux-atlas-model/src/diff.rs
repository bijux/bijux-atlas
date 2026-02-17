use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct GeneSignatureInput {
    pub gene_id: String,
    pub name: String,
    pub biotype: String,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub transcript_count: u64,
}

impl GeneSignatureInput {
    #[must_use]
    pub fn new(
        gene_id: String,
        name: String,
        biotype: String,
        seqid: String,
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
    pub gene_id: String,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub signature_sha256: String,
}

impl ReleaseGeneIndexEntry {
    #[must_use]
    pub fn new(
        gene_id: String,
        seqid: String,
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
            schema_version,
            dataset,
            entries,
        }
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
    pub gene_id: String,
    pub status: DiffStatus,
    pub seqid: Option<String>,
    pub start: Option<u64>,
    pub end: Option<u64>,
}

impl DiffRecord {
    #[must_use]
    pub fn new(
        gene_id: String,
        status: DiffStatus,
        seqid: Option<String>,
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
    pub from_release: String,
    pub to_release: String,
    pub species: String,
    pub assembly: String,
    pub scope: DiffScope,
    pub rows: Vec<DiffRecord>,
    pub next_cursor: Option<String>,
}

impl DiffPage {
    #[must_use]
    pub fn new(
        from_release: String,
        to_release: String,
        species: String,
        assembly: String,
        scope: DiffScope,
        rows: Vec<DiffRecord>,
        next_cursor: Option<String>,
    ) -> Self {
        Self {
            from_release,
            to_release,
            species,
            assembly,
            scope,
            rows,
            next_cursor,
        }
    }
}
