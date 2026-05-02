// SPDX-License-Identifier: Apache-2.0

pub use crate::domain::query::{
    classify_query, explain_query_plan, query_genes, BiotypePolicy, DuplicateGeneIdPolicy,
    DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy, GeneFields, GeneFilter,
    GeneNamePolicy, GeneQueryRequest, QueryLimits, RegionFilter, SeqidNormalizationPolicy,
    TranscriptIdPolicy, TranscriptTypePolicy, UnknownFeaturePolicy,
};

#[must_use]
pub fn estimate_work_units(request: &GeneQueryRequest) -> u64 {
    crate::domain::query::estimate_work_units(request)
}
