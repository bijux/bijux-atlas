// SPDX-License-Identifier: Apache-2.0

use serde_json::{Map, Value};

pub(super) fn sort_json_keys(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut sorted = Map::new();
            let mut keys = object.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                let nested = object.get(&key).cloned().expect("known key");
                sorted.insert(key, sort_json_keys(nested));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_json_keys).collect()),
        other => other,
    }
}
