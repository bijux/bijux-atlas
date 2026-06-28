// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionFilter {
    pub seqid: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneRow {
    pub gene_id: String,
    pub name: Option<String>,
    pub seqid: Option<String>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub biotype: Option<String>,
    pub transcript_count: Option<u64>,
    pub sequence_length: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneQueryResponse {
    pub rows: Vec<GeneRow>,
    pub next_cursor: Option<String>,
}
