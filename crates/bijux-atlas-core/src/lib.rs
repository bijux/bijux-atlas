// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod canonical;
pub mod error_codes;

#[cfg(feature = "serde")]
pub use canonical::{
    decode_cursor_payload, encode_cursor_payload, stable_json_bytes, stable_json_hash_hex,
    CanonicalError, CanonicalJson,
};
pub use canonical::{
    sha256, sha256_hex, stable_hash_bytes, stable_hash_hex, stable_sort_by_key, Hash256,
};
pub use error_codes::{ErrorCode, ERROR_CODES};
