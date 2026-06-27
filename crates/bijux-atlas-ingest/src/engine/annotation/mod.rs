// SPDX-License-Identifier: Apache-2.0

pub(crate) mod canonical_model;
pub(crate) mod extract;
pub(crate) mod normalized;

pub(crate) use canonical_model::{build_canonical_model, CanonicalModel};
pub(crate) use extract::{
    extract_gene_rows, parallelism_policy, ExonRecord, ExtractResult, GeneRecord, TranscriptRecord,
};
pub use normalized::ReplayCounts;
pub(crate) use normalized::{
    diff_normalized_record_ids, replay_counts_from_normalized, write_normalized_jsonl_zst,
};
