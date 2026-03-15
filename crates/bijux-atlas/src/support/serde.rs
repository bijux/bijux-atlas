// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Deserializer, Serializer};
use std::collections::BTreeMap;

pub mod hex_bytes {
    use super::*;

    pub fn serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut out = String::with_capacity(value.len() * 2);
        for b in value {
            use std::fmt::Write as _;
            let _ = write!(&mut out, "{b:02x}");
        }
        serializer.serialize_str(&out)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        if encoded.len() % 2 != 0 {
            return Err(serde::de::Error::custom("hex string must have even length"));
        }

        let mut out = Vec::with_capacity(encoded.len() / 2);
        let bytes = encoded.as_bytes();
        for i in (0..bytes.len()).step_by(2) {
            let pair = std::str::from_utf8(&bytes[i..i + 2])
                .map_err(|_| serde::de::Error::custom("hex string must be valid utf-8"))?;
            let byte = u8::from_str_radix(pair, 16)
                .map_err(|_| serde::de::Error::custom("hex string contains non-hex digits"))?;
            out.push(byte);
        }
        Ok(out)
    }
}

pub mod timestamp_string {
    use super::*;

    pub fn serialize<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.trim().is_empty() {
            return Ok(String::new());
        }
        if !value.contains('T') || !value.ends_with('Z') {
            return Err(serde::de::Error::custom(
                "timestamp must be RFC3339-like (e.g. 2026-02-24T00:00:00Z)",
            ));
        }
        Ok(value)
    }
}

#[must_use]
pub fn map_is_empty<K, V>(value: &BTreeMap<K, V>) -> bool {
    value.is_empty()
}
