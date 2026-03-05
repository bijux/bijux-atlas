// SPDX-License-Identifier: Apache-2.0

use crate::pagination::Page;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryFilter {
    pub gene_id: Option<String>,
    pub biotype: Option<String>,
    pub contig: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueryProjection {
    pub include_coords: bool,
    pub include_counts: bool,
    pub include_biotype: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetQuery {
    pub release: String,
    pub species: String,
    pub assembly: String,
    pub limit: u32,
    pub cursor: Option<String>,
    pub filter: QueryFilter,
    pub projection: QueryProjection,
}

impl DatasetQuery {
    #[must_use]
    pub fn new(
        release: impl Into<String>,
        species: impl Into<String>,
        assembly: impl Into<String>,
    ) -> Self {
        Self {
            release: release.into(),
            species: species.into(),
            assembly: assembly.into(),
            limit: 100,
            cursor: None,
            filter: QueryFilter::default(),
            projection: QueryProjection::default(),
        }
    }

    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    #[must_use]
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn with_gene_id(mut self, gene_id: impl Into<String>) -> Self {
        self.filter.gene_id = Some(gene_id.into());
        self
    }

    #[must_use]
    pub fn with_biotype(mut self, biotype: impl Into<String>) -> Self {
        self.filter.biotype = Some(biotype.into());
        self
    }

    #[must_use]
    pub fn with_contig(mut self, contig: impl Into<String>) -> Self {
        self.filter.contig = Some(contig.into());
        self
    }

    #[must_use]
    pub fn include_coords(mut self) -> Self {
        self.projection.include_coords = true;
        self
    }

    #[must_use]
    pub fn include_counts(mut self) -> Self {
        self.projection.include_counts = true;
        self
    }

    #[must_use]
    pub fn include_biotype(mut self) -> Self {
        self.projection.include_biotype = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryResult {
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamQuery {
    pub pages: Vec<Page<QueryResult>>,
}
