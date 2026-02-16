use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

#[must_use]
pub fn stable_sort_by_key<T, K: Ord, F: FnMut(&T) -> K>(mut values: Vec<T>, mut key: F) -> Vec<T> {
    values.sort_by_key(|v| key(v));
    values
}

pub fn stable_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let raw = serde_json::to_value(value)?;
    let normalized = normalize_json_value(raw);
    serde_json::to_vec(&normalized)
}

#[must_use]
pub fn stable_hash_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn stable_json_hash_hex<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let bytes = stable_json_bytes(value)?;
    Ok(stable_hash_hex(&bytes))
}

pub fn encode_cursor_payload<T: Serialize>(payload: &T) -> Result<String, serde_json::Error> {
    let bytes = stable_json_bytes(payload)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn decode_cursor_payload(token: &str) -> Result<Value, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|e| format!("cursor base64 decode failed: {e}"))?;
    serde_json::from_slice::<Value>(&bytes).map_err(|e| format!("cursor JSON decode failed: {e}"))
}

fn normalize_json_value(value: Value) -> Value {
    match value {
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

#[cfg(test)]
mod tests {
    use super::{stable_json_bytes, stable_json_hash_hex};
    use serde_json::json;

    #[test]
    fn canonical_json_orders_object_keys() {
        let value = json!({
            "z": 1,
            "a": {"d": 4, "b": 2},
            "arr": [{"k2": 2, "k1": 1}],
        });

        let bytes = stable_json_bytes(&value).expect("stable json bytes");
        let text = String::from_utf8(bytes).expect("utf8 json");
        assert_eq!(text, r#"{"a":{"b":2,"d":4},"arr":[{"k1":1,"k2":2}],"z":1}"#);
    }

    #[test]
    fn canonical_hash_is_deterministic_for_same_value() {
        let value = json!({"b": 2, "a": 1});
        let h1 = stable_json_hash_hex(&value).expect("hash 1");
        let h2 = stable_json_hash_hex(&value).expect("hash 2");
        assert_eq!(h1, h2);
    }
}
