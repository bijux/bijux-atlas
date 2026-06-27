// SPDX-License-Identifier: Apache-2.0

pub mod codes;
mod envelope;
mod mapping;

pub use codes::ApiErrorCode;
pub use envelope::{fallback_request_id, ApiError};
pub use mapping::{map_error, ApiErrorMapping, API_ERROR_SCHEMA_REF};
