// SPDX-License-Identifier: Apache-2.0

use super::filters::GeneQueryRequest;
use super::limits::QueryLimits;
use super::parser::{parse_gene_query, Predicate, SortKey};
use super::planner::{classify_ast, estimate_ast_cost, plan_query, PlanNode, QueryClass};
use super::query_error::{QueryError, QueryErrorCode};
use crate::domain::canonical;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryIntent {
    ExactIdLookup,
    IntervalLookup,
    FilteredDatasetScan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenQueryModel {
    pub schema_version: u64,
    pub intent: QueryIntent,
    pub class: QueryClass,
    pub plan_node: PlanNode,
    pub sort_key: SortKey,
    pub predicates: Vec<String>,
    pub estimated_work_units: u64,
    pub normalized_ast: String,
    pub query_contract_sha256: String,
}

pub fn freeze_query_model(
    req: &GeneQueryRequest,
    limits: &QueryLimits,
) -> Result<FrozenQueryModel, QueryError> {
    let ast = parse_gene_query(req)?;
    let plan = plan_query(&ast, limits)?;
    let intent = if ast
        .predicates
        .iter()
        .any(|p| matches!(p, Predicate::GeneId(_)))
    {
        QueryIntent::ExactIdLookup
    } else if ast
        .predicates
        .iter()
        .any(|p| matches!(p, Predicate::Region { .. }))
    {
        QueryIntent::IntervalLookup
    } else {
        QueryIntent::FilteredDatasetScan
    };

    let model = FrozenQueryModel {
        schema_version: 1,
        intent,
        class: classify_ast(&ast),
        plan_node: plan.node,
        sort_key: ast.sort_key,
        predicates: ast
            .predicates
            .iter()
            .map(|p| predicate_label(p).to_string())
            .collect(),
        estimated_work_units: estimate_ast_cost(&ast).work_units,
        normalized_ast: plan.normalized.clone(),
        query_contract_sha256: String::new(),
    };
    let bytes = canonical::stable_json_bytes(&model).map_err(|err| {
        QueryError::new(
            QueryErrorCode::Validation,
            format!("freeze query model serialization failed: {err}"),
        )
    })?;
    let mut out = model;
    out.query_contract_sha256 = canonical::stable_hash_hex(&bytes);
    Ok(out)
}

fn predicate_label(predicate: &Predicate) -> &'static str {
    match predicate {
        Predicate::GeneId(_) => "gene_id",
        Predicate::NameEquals(_) => "name",
        Predicate::NamePrefix(_) => "name_prefix",
        Predicate::Biotype(_) => "biotype",
        Predicate::Region { .. } => "region",
        Predicate::Strand(_) => "strand",
    }
}
