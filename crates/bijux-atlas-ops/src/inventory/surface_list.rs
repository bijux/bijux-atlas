// SPDX-License-Identifier: Apache-2.0

use crate::inventory::surface_registry::builtin_ops_registry;

pub fn build_surface_list_payload() -> serde_json::Value {
    let ops_registry = builtin_ops_registry();
    let domains = {
        let mut set = std::collections::BTreeSet::new();
        for entry in &ops_registry {
            set.insert(entry.domain);
        }
        set.into_iter().collect::<Vec<_>>()
    };

    serde_json::json!({
        "schema_version": 1,
        "generated_by": "bijux dev atlas ops generate surface-list --write-example",
        "status": "pass",
        "surfaces": ["check", "configs", "docs", "ops"],
        "crate_alignment": {
            "source": "cargo metadata",
            "status": "pass"
        },
        "ops_taxonomy": {
            "domains": domains,
            "entries": ops_registry.into_iter().map(|entry| {
                serde_json::json!({
                    "domain": entry.domain,
                    "verb": entry.verb,
                    "subverb": entry.subverb,
                    "tags": entry.tags.iter().map(|tag| format!("{tag:?}").to_ascii_lowercase()).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::build_surface_list_payload;

    #[test]
    fn surface_list_payload_reflects_owned_registry_taxonomy() {
        let payload = build_surface_list_payload();
        let domains = payload["ops_taxonomy"]["domains"]
            .as_array()
            .expect("domains array");
        let entries = payload["ops_taxonomy"]["entries"]
            .as_array()
            .expect("entries array");

        assert!(domains.iter().any(|value| value == "stack"));
        assert!(domains.iter().any(|value| value == "observe"));
        assert!(entries.iter().any(|entry| {
            entry["domain"] == "runbook"
                && entry["verb"] == "generate"
                && entry["subverb"].is_null()
        }));
    }
}
