// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod canonical;
pub mod error_codes;

pub use canonical::{
    sha256, sha256_hex, stable_hash_bytes, stable_hash_hex, stable_sort_by_key, Hash256,
};
#[cfg(feature = "serde")]
pub use canonical::{stable_json_bytes, stable_json_hash_hex};
pub use error_codes::{ErrorCode, ERROR_CODES};
