use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneFields {
    pub gene_id: bool,
    pub name: bool,
    pub coords: bool,
    pub biotype: bool,
    pub transcript_count: bool,
    pub sequence_length: bool,
}

impl Default for GeneFields {
    fn default() -> Self {
        Self {
            gene_id: true,
            name: true,
            coords: true,
            biotype: true,
            transcript_count: true,
            sequence_length: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionFilter {
    pub seqid: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GeneFilter {
    pub gene_id: Option<String>,
    pub name: Option<String>,
    pub name_prefix: Option<String>,
    pub biotype: Option<String>,
    pub region: Option<RegionFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneQueryRequest {
    pub fields: GeneFields,
    pub filter: GeneFilter,
    pub limit: usize,
    pub cursor: Option<String>,
    pub allow_full_scan: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TranscriptFilter {
    pub parent_gene_id: Option<String>,
    pub biotype: Option<String>,
    pub transcript_type: Option<String>,
    pub region: Option<RegionFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptQueryRequest {
    pub filter: TranscriptFilter,
    pub limit: usize,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptRow {
    pub transcript_id: String,
    pub parent_gene_id: String,
    pub transcript_type: String,
    pub biotype: Option<String>,
    pub seqid: String,
    pub start: u64,
    pub end: u64,
    pub exon_count: u64,
    pub total_exon_span: u64,
    pub cds_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptQueryResponse {
    pub rows: Vec<TranscriptRow>,
    pub next_cursor: Option<String>,
}

#[must_use]
pub fn compile_field_projection(fields: &GeneFields) -> Vec<String> {
    let mut select = vec!["g.gene_id".to_string()];
    select.push(if fields.name {
        "g.name".to_string()
    } else {
        "NULL AS name".to_string()
    });
    select.push(if fields.coords {
        "g.seqid".to_string()
    } else {
        "NULL AS seqid".to_string()
    });
    select.push(if fields.coords {
        "g.start".to_string()
    } else {
        "NULL AS start".to_string()
    });
    select.push(if fields.coords {
        "g.end".to_string()
    } else {
        "NULL AS end".to_string()
    });
    select.push(if fields.biotype {
        "g.biotype".to_string()
    } else {
        "NULL AS biotype".to_string()
    });
    select.push(if fields.transcript_count {
        "g.transcript_count".to_string()
    } else {
        "NULL AS transcript_count".to_string()
    });
    select.push(if fields.sequence_length {
        "g.sequence_length".to_string()
    } else {
        "NULL AS sequence_length".to_string()
    });
    select
}

#[must_use]
pub fn escape_like_prefix(prefix: &str) -> String {
    let mut out = String::with_capacity(prefix.len());
    for c in prefix.chars() {
        match c {
            '!' | '%' | '_' => {
                out.push('!');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

#[must_use]
pub fn normalize_name_lookup(input: &str) -> String {
    // Canonical query normalization policy: NFKC + Unicode lowercase.
    input.nfkc().collect::<String>().to_lowercase()
}
