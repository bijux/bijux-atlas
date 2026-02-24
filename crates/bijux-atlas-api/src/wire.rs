// SPDX-License-Identifier: Apache-2.0

use crate::convert::list_genes_response_dto;
use crate::dto::DatasetKeyDto;
use crate::errors::ApiError;
use crate::params::ListGenesParams;
use bijux_atlas_query::GeneQueryResponse;
use serde_json::Value;
use std::collections::BTreeSet;

pub trait QueryAdapter {
    fn list_genes(&self, params: &ListGenesParams) -> Result<GeneQueryResponse, ApiError>;
}

pub fn list_genes_v1<A: QueryAdapter>(
    adapter: &A,
    params: &ListGenesParams,
) -> Result<Value, ApiError> {
    let page = adapter.list_genes(params)?;
    let requested = params
        .include
        .as_ref()
        .map(|v| v.iter().cloned().collect::<BTreeSet<_>>());
    let dataset = DatasetKeyDto::new(
        params.release.clone(),
        params.species.clone(),
        params.assembly.clone(),
    )
    .map_err(|reason| {
        ApiError::validation_failed(serde_json::json!([{ "field": "dataset", "reason": reason }]))
    })?;
    let dto = list_genes_response_dto(page, dataset, requested.as_ref())?;
    serde_json::to_value(dto).map_err(|e| {
        ApiError::validation_failed(
            serde_json::json!([{ "field": "response", "reason": e.to_string() }]),
        )
    })
}
