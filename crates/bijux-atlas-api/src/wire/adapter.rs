// SPDX-License-Identifier: Apache-2.0

use crate::errors::ApiError;
use crate::params::ListGenesParams;
use bijux_atlas_model::query::GeneQueryResponse;

pub trait QueryAdapter {
    fn list_genes(&self, params: &ListGenesParams) -> Result<GeneQueryResponse, ApiError>;
}
