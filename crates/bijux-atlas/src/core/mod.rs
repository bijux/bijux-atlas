// SPDX-License-Identifier: Apache-2.0

use crate::contracts::errors::Error;
use crate::contracts::errors::Result;

pub use bijux_atlas_core::canonical::{
    sha256, sha256_hex, stable_hash_bytes, stable_hash_hex, stable_sort_by_key, Hash256,
};
pub type CanonicalJson = bijux_atlas_core::canonical::CanonicalJson;

pub fn stable_json_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    bijux_atlas_core::canonical::stable_json_bytes(value)
        .map_err(|err| Error::json_encoding(err.to_string()))
}

pub fn stable_json_hash_hex<T: serde::Serialize>(value: &T) -> Result<String> {
    bijux_atlas_core::canonical::stable_json_hash_hex(value)
        .map_err(|err| Error::json_encoding(err.to_string()))
}

pub fn encode_cursor_payload<T: serde::Serialize>(payload: &T) -> Result<String> {
    bijux_atlas_core::canonical::encode_cursor_payload(payload)
        .map_err(|err| Error::json_encoding(err.to_string()))
}

pub fn decode_cursor_payload(token: &str) -> Result<serde_json::Value> {
    bijux_atlas_core::canonical::decode_cursor_payload(token).map_err(|err| match err {
        bijux_atlas_core::canonical::CanonicalError::Base64(inner) => {
            Error::DecodeCursorBase64(inner.to_string())
        }
        bijux_atlas_core::canonical::CanonicalError::Json(inner) => {
            Error::DecodeCursorJson(inner.to_string())
        }
    })
}
