    if !ctx.adapters.fs.exists(ctx.repo_root, authority_index_rel) {
        violations.push(violation(
            "OPS_INVENTORY_AUTHORITY_INDEX_MISSING",
            "missing ops/inventory/authority-index.json".to_string(),
            "restore authority-index.json with authoritative and generated registry roles",
            Some(authority_index_rel),
        ));
    } else {
        let authority_index_text = fs::read_to_string(ctx.repo_root.join(authority_index_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let authority_index_json: serde_json::Value = serde_json::from_str(&authority_index_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;

        let hierarchy = authority_index_json
            .get("authority_hierarchy")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let expected_hierarchy = ["ops/inventory", "ops/schema", "docs"];
        for (position, expected) in expected_hierarchy.iter().enumerate() {
            let actual = hierarchy
                .get(position)
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if actual != *expected {
                violations.push(violation(
                    "OPS_INVENTORY_AUTHORITY_HIERARCHY_DRIFT",
                    format!(
                        "authority_hierarchy[{position}] must be `{expected}` but found `{actual}`"
                    ),
                    "keep authority hierarchy stable as ops/inventory -> ops/schema -> docs",
                    Some(authority_index_rel),
                ));
            }
        }

        let entries = authority_index_json
            .get("authoritative_files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut has_contracts_map_authoritative = false;
        let mut has_contracts_generated = false;
        for entry in &entries {
            let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
            let role = entry.get("role").and_then(|v| v.as_str()).unwrap_or_default();
            let consumer = entry
                .get("consumer")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .trim();
            if consumer.is_empty() {
                violations.push(violation(
                    "OPS_INVENTORY_AUTHORITY_INDEX_CONSUMER_MISSING",
                    format!("authority-index entry `{path}` must include non-empty `consumer`"),
                    "add authority-index authoritative_files[].consumer with the enforcing check or runtime consumer id",
                    Some(authority_index_rel),
                ));
            }
            if role == "generated" {
                let producer = entry
                    .get("producer")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .trim();
                if producer.is_empty() {
                    violations.push(violation(
                        "OPS_INVENTORY_AUTHORITY_INDEX_PRODUCER_MISSING",
                        format!("generated authority-index entry `{path}` must include non-empty `producer`"),
                        "add authority-index authoritative_files[].producer for generated entries",
                        Some(authority_index_rel),
                    ));
                }
            }
            if path == "ops/inventory/contracts-map.json" && role == "authoritative" {
                has_contracts_map_authoritative = true;
            }
            if path == "ops/inventory/contracts.json" && role == "generated" {
                let derived_from = entry
                    .get("derived_from")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if derived_from == "ops/inventory/contracts-map.json" {
                    has_contracts_generated = true;
                }
            }
        }
        if !has_contracts_map_authoritative {
            violations.push(violation(
                "OPS_INVENTORY_AUTHORITY_INDEX_CONTRACTS_MAP_ROLE_MISSING",
                "authority-index must mark ops/inventory/contracts-map.json as authoritative"
                    .to_string(),
                "add contracts-map authoritative role to authority-index",
                Some(authority_index_rel),
            ));
        }
        if !has_contracts_generated {
            violations.push(violation(
                "OPS_INVENTORY_AUTHORITY_INDEX_CONTRACTS_MIRROR_ROLE_MISSING",
                "authority-index must mark ops/inventory/contracts.json as generated from contracts-map"
                    .to_string(),
                "set contracts.json role=generated and derived_from=ops/inventory/contracts-map.json",
                Some(authority_index_rel),
            ));
        }

        if !ctx
            .adapters
            .fs
            .exists(ctx.repo_root, authoritative_file_list_rel)
        {
            violations.push(violation(
                "OPS_INVENTORY_AUTHORITATIVE_FILE_LIST_MISSING",
                "missing ops/inventory/authoritative-file-list.json".to_string(),
                "restore authoritative-file-list.json with authoritative and derived path indexes",
                Some(authoritative_file_list_rel),
            ));
        } else {
            let file_list_text =
                fs::read_to_string(ctx.repo_root.join(authoritative_file_list_rel))
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
            let file_list_json: serde_json::Value = serde_json::from_str(&file_list_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let authoritative_paths = file_list_json
                .get("authoritative_paths")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(ToString::to_string)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default();
            let derived_paths = file_list_json
                .get("derived_paths")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(ToString::to_string)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default();
            if !authoritative_paths.contains("ops/inventory/contracts-map.json") {
                violations.push(violation(
                    "OPS_INVENTORY_AUTHORITATIVE_FILE_LIST_CONTRACTS_MAP_MISSING",
                    "authoritative-file-list must include ops/inventory/contracts-map.json"
                        .to_string(),
                    "add contracts-map to authoritative_paths in authoritative-file-list",
                    Some(authoritative_file_list_rel),
                ));
            }
            for entry in &entries {
                let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
                let role = entry.get("role").and_then(|v| v.as_str()).unwrap_or_default();
                if role == "authoritative" && !authoritative_paths.contains(path) {
                    violations.push(violation(
                        "OPS_INVENTORY_AUTHORITATIVE_FILE_LIST_DRIFT",
                        format!(
                            "authority-index authoritative path `{path}` missing from authoritative-file-list"
                        ),
                        "sync authoritative-file-list authoritative_paths with authority-index entries",
                        Some(authoritative_file_list_rel),
                    ));
                }
                if role == "generated" && !derived_paths.contains(path) {
                    violations.push(violation(
                        "OPS_INVENTORY_DERIVED_FILE_LIST_DRIFT",
                        format!(
                            "authority-index generated path `{path}` missing from authoritative-file-list derived_paths"
                        ),
                        "sync authoritative-file-list derived_paths with authority-index generated entries",
                        Some(authoritative_file_list_rel),
                    ));
                }
            }
        }
    }
