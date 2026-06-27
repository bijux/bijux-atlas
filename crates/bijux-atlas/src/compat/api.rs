// SPDX-License-Identifier: Apache-2.0

pub const CRATE_NAME: &str = bijux_atlas_api::CRATE_NAME;
pub const API_POLICY_LATEST_ALIAS: &str = bijux_atlas_api::API_POLICY_LATEST_ALIAS;
pub const API_POLICY_NO_IMPLICIT_DEFAULT_DATASET: &str =
    bijux_atlas_api::API_POLICY_NO_IMPLICIT_DEFAULT_DATASET;

pub mod compat {
    pub use bijux_atlas_api::compat::*;
}

pub mod convert {
    pub use bijux_atlas_api::convert::*;
}

pub mod dto {
    pub use bijux_atlas_api::dto::*;
}

pub mod error_mapping {
    pub use bijux_atlas_api::error_mapping::*;
}

pub mod errors {
    pub use bijux_atlas_api::errors::*;
}

pub mod generated {
    pub mod error_codes {
        pub use bijux_atlas_api::generated::error_codes::*;
    }
}

pub mod openapi {
    pub use bijux_atlas_api::openapi::*;
}

pub mod params {
    pub use bijux_atlas_api::params::*;
}

pub mod responses {
    pub use bijux_atlas_api::responses::*;
}

pub mod wire {
    pub use bijux_atlas_api::wire::*;
}

pub use bijux_atlas_api::{
    dataset_route_key, fallback_request_id, openapi_v1_spec, parse_list_genes_params,
    parse_list_genes_params_with_limit, parse_range_filter, parse_region_filter, ApiContentType,
    ApiError, ApiErrorCode, ApiResponseEnvelope, ContentNegotiation, DatasetKeyDto, IncludeField,
    ListGenesParams, QueryAdapter, MAX_CURSOR_BYTES,
};
