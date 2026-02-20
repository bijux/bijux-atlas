#![deny(clippy::redundant_clone)]

use crate::*;
use bijux_atlas_model::ShardCatalog;
use bijux_atlas_query::{
    estimate_work_units, prepared_sql_for_class_export, query_gene_by_id_fast,
    query_gene_id_name_json_minimal_fast, query_genes_fanout, select_shards_for_request,
};
use serde_json::json;
use tracing::{info, info_span, warn};

#[path = "genes/admission.rs"]
mod genes_admission;
#[path = "genes/response.rs"]
mod genes_response;


include!("genes/handler.rs");
