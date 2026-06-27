// SPDX-License-Identifier: Apache-2.0

use sha2::{Digest, Sha256};

#[cfg(feature = "serde")]
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
#[cfg(feature = "serde")]
use base64::Engine;
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use serde_json::{Map, Value};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    #[must_use]
    pub(crate) const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        let mut out = String::with_capacity(64);
        for b in self.0 {
            use std::fmt::Write as _;
            let _ = write!(&mut out, "{b:02x}");
        }
        out
    }
}

impl core::fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Hash256({})", self.to_hex())
    }
}

impl core::fmt::Display for Hash256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[must_use]
pub fn stable_sort_by_key<T, K: Ord, F: FnMut(&T) -> K>(mut values: Vec<T>, mut key: F) -> Vec<T> {
    values.sort_by_key(|v| key(v));
    values
}

#[must_use]
pub fn stable_hash_bytes(bytes: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest);
    Hash256::from_bytes(out)
}

#[must_use]
pub fn stable_hash_hex(bytes: &[u8]) -> String {
    stable_hash_bytes(bytes).to_hex()
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    stable_hash_hex(bytes)
}

#[must_use]
pub fn sha256(bytes: &[u8]) -> Hash256 {
    stable_hash_bytes(bytes)
}

#[cfg(feature = "serde")]
#[derive(Debug)]
pub enum CanonicalError {
    Base64(base64::DecodeError),
    Json(serde_json::Error),
}

#[cfg(feature = "serde")]
impl core::fmt::Display for CanonicalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Base64(err) => write!(f, "base64 decode failed: {err}"),
            Self::Json(err) => write!(f, "json encoding failed: {err}"),
        }
    }
}

#[cfg(feature = "serde")]
impl std::error::Error for CanonicalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Base64(err) => Some(err),
            Self::Json(err) => Some(err),
        }
    }
}

#[cfg(feature = "serde")]
impl From<base64::DecodeError> for CanonicalError {
    fn from(value: base64::DecodeError) -> Self {
        Self::Base64(value)
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for CanonicalError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
pub struct CanonicalJson(Value);

#[cfg(feature = "serde")]
impl CanonicalJson {
    pub fn from_serialize<T: Serialize>(value: &T) -> Result<Self, CanonicalError> {
        Ok(Self(normalize_json_value(serde_json::to_value(value)?)))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CanonicalError> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn hash(&self) -> Result<Hash256, CanonicalError> {
        let bytes = self.to_bytes()?;
        Ok(stable_hash_bytes(&bytes))
    }
}

#[cfg(feature = "serde")]
pub fn stable_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    CanonicalJson::from_serialize(value)?.to_bytes()
}

#[cfg(feature = "serde")]
#[must_use]
pub fn stable_json_hash_hex<T: Serialize>(value: &T) -> Result<String, CanonicalError> {
    Ok(CanonicalJson::from_serialize(value)?.hash()?.to_hex())
}

#[cfg(feature = "serde")]
pub fn encode_cursor_payload<T: Serialize>(payload: &T) -> Result<String, CanonicalError> {
    let bytes = stable_json_bytes(payload)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

#[cfg(feature = "serde")]
pub fn decode_cursor_payload(token: &str) -> Result<Value, CanonicalError> {
    let bytes = URL_SAFE_NO_PAD.decode(token)?;
    Ok(serde_json::from_slice::<Value>(&bytes)?)
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
