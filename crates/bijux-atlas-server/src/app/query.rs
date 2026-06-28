// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_model::ShardCatalog;
pub use bijux_atlas_query::{
    classify_query, explain_query_plan, query_genes, BiotypePolicy, DuplicateGeneIdPolicy,
    DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy, GeneFields, GeneFilter, GeneNamePolicy,
    GeneQueryRequest, IntervalSemantics, QueryLimits, QuerySort, RegionFilter,
    SeqidNormalizationPolicy, StrandMode, TranscriptIdPolicy, TranscriptTypePolicy,
    UnknownFeaturePolicy,
};
use rusqlite::Connection;

#[must_use]
pub fn estimate_work_units(request: &GeneQueryRequest) -> u64 {
    bijux_atlas_query::estimate_work_units(request)
}

pub fn prepared_sql_for_class(query_class: bijux_atlas_query::QueryClass) -> &'static str {
    bijux_atlas_query::prepared_sql_for_class_export(query_class)
}

pub fn query_gene_by_id(
    conn: &Connection,
    gene_id: &str,
    fields: &GeneFields,
) -> Result<Option<bijux_atlas_query::GeneRow>, String> {
    bijux_atlas_query::query_gene_by_id_fast(conn, gene_id, fields).map_err(|err| err.to_string())
}

pub fn query_gene_id_name_json_minimal(
    conn: &Connection,
    gene_id: &str,
) -> Result<Option<Vec<u8>>, String> {
    bijux_atlas_query::query_gene_id_name_json_minimal_fast(conn, gene_id)
        .map_err(|err| err.to_string())
}

pub fn query_genes_fanout_execute(
    shards: &[&Connection],
    req: &GeneQueryRequest,
    limits: &QueryLimits,
    cursor_secret: &[u8],
) -> Result<bijux_atlas_query::GeneQueryResponse, String> {
    bijux_atlas_query::query_genes_fanout(shards, req, limits, cursor_secret)
        .map_err(|err| err.to_string())
}

pub fn select_shards(req: &GeneQueryRequest, catalog: &ShardCatalog) -> Vec<String> {
    bijux_atlas_query::select_shards_for_request(req, catalog)
}
