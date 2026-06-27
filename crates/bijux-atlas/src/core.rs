// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "serde")]
use crate::contracts::errors::Error;
use crate::contracts::errors::Result;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use serde_json::{Map, Value};

pub use bijux_atlas_core::canonical::{
    sha256, sha256_hex, stable_hash_bytes, stable_hash_hex, stable_sort_by_key, Hash256,
};

#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
pub struct CanonicalJson(Value);

#[cfg(feature = "serde")]
impl CanonicalJson {
    pub fn from_serialize<T: Serialize>(value: &T) -> Result<Self> {
        let raw =
            serde_json::to_value(value).map_err(|err| Error::json_encoding(err.to_string()))?;
        Ok(Self(normalize_json_value(raw)))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.0).map_err(|err| Error::json_encoding(err.to_string()))
    }

    pub fn hash(&self) -> Result<Hash256> {
        let bytes = self.to_bytes()?;
        Ok(stable_hash_bytes(&bytes))
    }
}

#[cfg(feature = "serde")]
pub fn stable_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    CanonicalJson::from_serialize(value)?.to_bytes()
}

#[cfg(feature = "serde")]
pub fn stable_json_hash_hex<T: Serialize>(value: &T) -> Result<String> {
    Ok(CanonicalJson::from_serialize(value)?.hash()?.to_hex())
}

#[cfg(feature = "serde")]
pub fn encode_cursor_payload<T: Serialize>(payload: &T) -> Result<String> {
    let bytes = stable_json_bytes(payload)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

#[cfg(feature = "serde")]
pub fn decode_cursor_payload(token: &str) -> Result<Value> {
    let bytes = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|e| Error::DecodeCursorBase64(e.to_string()))?;
    serde_json::from_slice::<Value>(&bytes).map_err(|e| Error::DecodeCursorJson(e.to_string()))
}

#[cfg(feature = "serde")]
fn normalize_json_value(value: Value) -> Value {
    match value {
        Value::Number(n) => Value::Number(normalize_json_number(n)),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut entries: Vec<(String, Value)> = map
                .into_iter()
                .map(|(k, v)| (k, normalize_json_value(v)))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            for (k, v) in entries {
                sorted.insert(k, v);
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(normalize_json_value).collect()),
        other => other,
    }
}

#[cfg(feature = "serde")]
fn normalize_json_number(number: serde_json::Number) -> serde_json::Number {
    if let Some(value) = number.as_f64() {
        if value == 0.0 {
            return serde_json::Number::from(0);
        }
    }
    number
}
