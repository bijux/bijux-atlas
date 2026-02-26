pub(super) fn check_ops_inventory_contract_integrity(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let contracts_map_rel = Path::new("ops/inventory/contracts-map.json");
    let contracts_rel = Path::new("ops/inventory/contracts.json");
    let authority_index_rel = Path::new("ops/inventory/authority-index.json");
    let authoritative_file_list_rel = Path::new("ops/inventory/authoritative-file-list.json");
    let contracts_meta_rel = Path::new("ops/inventory/meta/contracts.json");
    let namespaces_rel = Path::new("ops/inventory/namespaces.json");
    let layers_rel = Path::new("ops/inventory/layers.json");
    let gates_rel = Path::new("ops/inventory/gates.json");
    let drill_links_rel = Path::new("ops/inventory/drill-contract-links.json");
    let control_graph_rel = Path::new("ops/inventory/control-graph.json");
    let surfaces_rel = Path::new("ops/inventory/surfaces.json");
    let owners_rel = Path::new("ops/inventory/owners.json");
    let policy_rel = Path::new("ops/inventory/policies/dev-atlas-policy.json");
    let policy_schema_rel = Path::new("ops/inventory/policies/dev-atlas-policy.schema.json");
    let pins_rel = Path::new("ops/inventory/pins.yaml");
    let stack_manifest_rel = Path::new("ops/stack/generated/version-manifest.json");
    let stack_toml_rel = Path::new("ops/stack/stack.toml");
    let stack_dependency_graph_rel = Path::new("ops/stack/generated/dependency-graph.json");
    let stack_service_contract_rel = Path::new("ops/stack/service-dependency-contract.json");
    let stack_evolution_policy_rel = Path::new("ops/stack/evolution-policy.json");
    let k8s_install_matrix_rel = Path::new("ops/k8s/install-matrix.json");
    let k8s_rollout_contract_rel = Path::new("ops/k8s/rollout-safety-contract.json");
    let registry_rel = Path::new("ops/inventory/registry.toml");
    let tools_rel = Path::new("ops/inventory/tools.toml");
    let inventory_index_rel = Path::new("ops/_generated.example/inventory-index.json");
    let stack_drift_rel = Path::new("ops/_generated.example/stack-drift-report.json");

    let contracts_map_text = fs::read_to_string(ctx.repo_root.join(contracts_map_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_map: serde_json::Value = serde_json::from_str(&contracts_map_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if contracts_map.get("authoritative").and_then(|v| v.as_bool()) != Some(true) {
        violations.push(violation(
            "OPS_INVENTORY_CONTRACTS_MAP_NOT_AUTHORITATIVE",
            "ops/inventory/contracts-map.json must declare `authoritative: true`".to_string(),
            "mark contracts-map as the authoritative inventory contract manifest",
            Some(contracts_map_rel),
        ));
    }
    let items = contracts_map
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut seen_paths = std::collections::BTreeSet::new();
    let mut item_paths = std::collections::BTreeSet::new();
    for item in &items {
        let Some(path) = item.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let path_buf = PathBuf::from(path);
        if !seen_paths.insert(path.to_string()) {
            violations.push(violation(
                "OPS_INVENTORY_CONTRACTS_MAP_DUPLICATE_PATH",
                format!("duplicate contracts-map item path `{path}`"),
                "keep unique paths in contracts-map items",
                Some(contracts_map_rel),
            ));
        }
        item_paths.insert(path_buf.clone());
        if !ctx.adapters.fs.exists(ctx.repo_root, &path_buf) {
            violations.push(violation(
                "OPS_INVENTORY_CONTRACTS_MAP_PATH_MISSING",
                format!("contracts-map references missing path `{path}`"),
                "remove stale path or restore referenced inventory artifact",
                Some(contracts_map_rel),
            ));
        }
        let schema = item
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        if schema != "none" && !ctx.adapters.fs.exists(ctx.repo_root, Path::new(schema)) {
            violations.push(violation(
                "OPS_INVENTORY_SCHEMA_REFERENCE_MISSING",
                format!("contracts-map references missing schema `{schema}` for `{path}`"),
                "restore schema path or fix schema pointer in contracts-map",
                Some(contracts_map_rel),
            ));
        }
        let consumer = item
            .get("consumer")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim();
        if consumer.is_empty() {
            violations.push(violation(
                "OPS_INVENTORY_CONTRACTS_MAP_CONSUMER_MISSING",
                format!("contracts-map item `{path}` is missing non-empty `consumer`"),
                "add contracts-map.items[].consumer with the enforcing check or runtime consumer id",
                Some(contracts_map_rel),
            ));
        }
    }

    let contracts_text = fs::read_to_string(ctx.repo_root.join(contracts_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contracts_json: serde_json::Value =
        serde_json::from_str(&contracts_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if contracts_json
        .get("generated_from")
        .and_then(|v| v.as_str())
        != Some("ops/inventory/contracts-map.json")
    {
        violations.push(violation(
            "OPS_INVENTORY_CONTRACTS_GENERATION_METADATA_MISSING",
            "ops/inventory/contracts.json must declare `generated_from: ops/inventory/contracts-map.json`"
                .to_string(),
            "mark contracts.json as a generated mirror of contracts-map",
            Some(contracts_rel),
        ));
    }
    if !item_paths.contains(authority_index_rel) {
        violations.push(violation(
            "OPS_INVENTORY_AUTHORITY_INDEX_NOT_REGISTERED",
            "ops/inventory/authority-index.json must be declared in contracts-map items".to_string(),
            "register ops/inventory/authority-index.json in contracts-map with schema and consumer metadata",
            Some(contracts_map_rel),
        ));
    }
    if !item_paths.contains(authoritative_file_list_rel) {
        violations.push(violation(
            "OPS_INVENTORY_AUTHORITATIVE_FILE_LIST_NOT_REGISTERED",
            "ops/inventory/authoritative-file-list.json must be declared in contracts-map items"
                .to_string(),
            "register ops/inventory/authoritative-file-list.json in contracts-map with schema and consumer metadata",
            Some(contracts_map_rel),
        ));
    }
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
    if ctx.adapters.fs.exists(ctx.repo_root, contracts_meta_rel) {
        violations.push(violation(
            "OPS_INVENTORY_DUPLICATE_CONTRACT_REGISTRY_FORBIDDEN",
            "legacy duplicate contract registry `ops/inventory/meta/contracts.json` is forbidden"
                .to_string(),
            "remove ops/inventory/meta/contracts.json and keep ops/inventory/contracts.json as the single contracts registry",
            Some(contracts_meta_rel),
        ));
    }

    let inventory_root = ctx.repo_root.join("ops/inventory");
    let allowed_unmapped = [
        PathBuf::from("ops/inventory/OWNER.md"),
        PathBuf::from("ops/inventory/README.md"),
        PathBuf::from("ops/inventory/REQUIRED_FILES.md"),
        PathBuf::from("ops/inventory/registry.toml"),
        PathBuf::from("ops/inventory/tools.toml"),
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    for file in walk_files(&inventory_root) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.starts_with(Path::new("ops/inventory/contracts/"))
            || rel.starts_with(Path::new("ops/inventory/meta/"))
            || rel.starts_with(Path::new("ops/inventory/policies/"))
        {
            continue;
        }
        if allowed_unmapped.contains(rel) {
            continue;
        }
        if !item_paths.contains(rel) {
            violations.push(violation(
                "OPS_INVENTORY_ORPHAN_FILE",
                format!(
                    "orphan inventory file not tracked by contracts-map: `{}`",
                    rel.display()
                ),
                "add file to contracts-map or remove orphan artifact",
                Some(rel),
            ));
        }
    }

    let namespaces_text = fs::read_to_string(ctx.repo_root.join(namespaces_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let namespaces_json: serde_json::Value = serde_json::from_str(&namespaces_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let namespace_keys = namespaces_json
        .get("namespaces")
        .and_then(|v| v.as_object())
        .map(|v| v.keys().cloned().collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();

    let layers_text = fs::read_to_string(ctx.repo_root.join(layers_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let layers_json: serde_json::Value =
        serde_json::from_str(&layers_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if layers_text.contains("\"obs\"") {
        violations.push(violation(
            "OPS_INVENTORY_LAYER_LEGACY_OBS_REFERENCE",
            "ops/inventory/layers.json contains legacy `obs` references".to_string(),
            "replace `obs` with canonical `observe` layer naming",
            Some(layers_rel),
        ));
    }
    let layer_namespace_keys = layers_json
        .get("namespaces")
        .and_then(|v| v.as_object())
        .map(|v| v.keys().cloned().collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();
    if namespace_keys != layer_namespace_keys {
        violations.push(violation(
            "OPS_INVENTORY_NAMESPACE_LAYER_DRIFT",
            format!(
                "namespace key mismatch between namespaces.json and layers.json: namespaces={namespace_keys:?} layers={layer_namespace_keys:?}"
            ),
            "keep namespace keys synchronized between namespaces and layer policy",
            Some(namespaces_rel),
        ));
    }

    let gates_text = fs::read_to_string(ctx.repo_root.join(gates_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let gates_json: serde_json::Value =
        serde_json::from_str(&gates_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_text = fs::read_to_string(ctx.repo_root.join(surfaces_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let surfaces_json: serde_json::Value =
        serde_json::from_str(&surfaces_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let action_ids = surfaces_json
        .get("actions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let gate_ids = gates_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|gate| gate.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();

    let control_graph_text = fs::read_to_string(ctx.repo_root.join(control_graph_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let control_graph_json: serde_json::Value = serde_json::from_str(&control_graph_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let nodes = control_graph_json
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let edges = control_graph_json
        .get("edges")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut node_ids = std::collections::BTreeSet::new();
    for node in &nodes {
        let id = node.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let node_type = node.get("type").and_then(|v| v.as_str()).unwrap_or_default();
        if id.is_empty() || !node_ids.insert(id.to_string()) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_NODE_INVALID",
                format!("control graph node id is empty or duplicated: `{id}`"),
                "ensure control-graph nodes have unique non-empty ids",
                Some(control_graph_rel),
            ));
        }
        if let Some(path) = node.get("path").and_then(|v| v.as_str()) {
            if !ctx.adapters.fs.exists(ctx.repo_root, Path::new(path)) {
                violations.push(violation(
                    "OPS_INVENTORY_CONTROL_GRAPH_NODE_PATH_MISSING",
                    format!("control graph node path does not exist: `{path}`"),
                    "fix node.path to an existing ops artifact path",
                    Some(control_graph_rel),
                ));
            }
        }
        if node_type == "action" {
            let action_id = id.strip_prefix("action.").unwrap_or(id);
            if !action_ids.contains(action_id) {
                violations.push(violation(
                    "OPS_INVENTORY_CONTROL_GRAPH_ACTION_NOT_IN_SURFACES",
                    format!("control graph action node `{id}` is not defined in surfaces actions"),
                    "align action nodes with ops/inventory/surfaces.json action ids",
                    Some(control_graph_rel),
                ));
            }
        }
        if node_type == "gate" {
            let gate_id = id.strip_prefix("gate.").unwrap_or(id);
            if !gate_ids.contains(gate_id) {
                violations.push(violation(
                    "OPS_INVENTORY_CONTROL_GRAPH_GATE_NOT_IN_GATES",
                    format!("control graph gate node `{id}` is not defined in gates.json"),
                    "align gate nodes with ops/inventory/gates.json ids",
                    Some(control_graph_rel),
                ));
            }
        }
    }
    let mut seen_kinds = std::collections::BTreeSet::new();
    let mut adjacency = std::collections::BTreeMap::<String, Vec<String>>::new();
    for edge in &edges {
        let from = edge.get("from").and_then(|v| v.as_str()).unwrap_or_default();
        let to = edge.get("to").and_then(|v| v.as_str()).unwrap_or_default();
        let kind = edge.get("kind").and_then(|v| v.as_str()).unwrap_or_default();
        if !node_ids.contains(from) || !node_ids.contains(to) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_EDGE_REFERENCE_INVALID",
                format!("control graph edge references unknown node: from=`{from}` to=`{to}`"),
                "ensure edge endpoints reference existing node ids",
                Some(control_graph_rel),
            ));
            continue;
        }
        seen_kinds.insert(kind.to_string());
        if matches!(
            kind,
            "dependency" | "consumer" | "producer" | "lifecycle" | "drift"
        ) {
            adjacency
                .entry(from.to_string())
                .or_default()
                .push(to.to_string());
        }
    }
    for required_kind in ["dependency", "consumer", "producer", "lifecycle", "drift"] {
        if !seen_kinds.contains(required_kind) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_EDGE_KIND_MISSING",
                format!("control graph is missing required edge kind `{required_kind}`"),
                "add at least one edge for each required control graph edge kind",
                Some(control_graph_rel),
            ));
        }
    }
    fn has_cycle(
        node: &str,
        adjacency: &std::collections::BTreeMap<String, Vec<String>>,
        visiting: &mut std::collections::BTreeSet<String>,
        visited: &mut std::collections::BTreeSet<String>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }
        visiting.insert(node.to_string());
        if let Some(neighbors) = adjacency.get(node) {
            for next in neighbors {
                if has_cycle(next, adjacency, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
        false
    }
    let mut visiting = std::collections::BTreeSet::new();
    let mut visited = std::collections::BTreeSet::new();
    for node in node_ids.iter() {
        if has_cycle(node, &adjacency, &mut visiting, &mut visited) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_CYCLE_DETECTED",
                "control graph contains a cycle in dependency/consumer/producer/lifecycle/drift edges"
                    .to_string(),
                "break cyclic control graph edges to keep inventory graph acyclic",
                Some(control_graph_rel),
            ));
            break;
        }
    }

    let drills_text = fs::read_to_string(ctx.repo_root.join(Path::new("ops/inventory/drills.json")))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let drills_json: serde_json::Value =
        serde_json::from_str(&drills_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let drill_ids = drills_json
        .get("drills")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let drill_links_text = fs::read_to_string(ctx.repo_root.join(drill_links_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let drill_links_json: serde_json::Value = serde_json::from_str(&drill_links_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let linked_drills = drill_links_json
        .get("links")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("drill_id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    for drill_id in &drill_ids {
        if !linked_drills.contains(drill_id) {
            violations.push(violation(
                "OPS_INVENTORY_DRILL_CONTRACT_LINK_MISSING",
                format!("drill id `{drill_id}` has no mapping in drill-contract-links"),
                "add drill-contract-links entry for each drill id with at least one contract path",
                Some(drill_links_rel),
            ));
        }
    }
    if let Some(links) = drill_links_json.get("links").and_then(|v| v.as_array()) {
        for link in links {
            let drill_id = link
                .get("drill_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !drill_ids.contains(drill_id) {
                violations.push(violation(
                    "OPS_INVENTORY_DRILL_CONTRACT_LINK_UNKNOWN_DRILL",
                    format!("drill-contract-links references unknown drill id `{drill_id}`"),
                    "remove stale drill link entries or restore missing drill ids",
                    Some(drill_links_rel),
                ));
            }
            for contract_path in link
                .get("contract_paths")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item.as_str())
            {
                if !ctx
                    .adapters
                    .fs
                    .exists(ctx.repo_root, Path::new(contract_path))
                {
                    violations.push(violation(
                        "OPS_INVENTORY_DRILL_CONTRACT_LINK_PATH_MISSING",
                        format!(
                            "drill-contract-links references missing contract path `{contract_path}`"
                        ),
                        "fix contract_paths to existing ops domain CONTRACT.md files",
                        Some(drill_links_rel),
                    ));
                }
            }
        }
    }
    if let Some(gates) = gates_json.get("gates").and_then(|v| v.as_array()) {
        let gate_ids = gates
            .iter()
            .filter_map(|gate| gate.get("id").and_then(|v| v.as_str()))
            .collect::<std::collections::BTreeSet<_>>();
        let required_release_gates = [
            "ops.gate.ssot",
            "ops.gate.validate",
            "ops.gate.structure",
            "ops.gate.docs",
            "ops.gate.generated",
            "ops.gate.evidence",
            "ops.gate.fixtures",
            "ops.gate.naming",
            "ops.gate.inventory",
            "ops.gate.schema",
        ];
        for required_gate in required_release_gates {
            if !gate_ids.contains(required_gate) {
                violations.push(violation(
                    "OPS_INVENTORY_RELEASE_GATE_MISSING",
                    format!("required release gate id missing from gates.json: `{required_gate}`"),
                    "add the missing release gate id to ops/inventory/gates.json",
                    Some(gates_rel),
                ));
            }
        }
        for gate in gates {
            let Some(action_id) = gate.get("action_id").and_then(|v| v.as_str()) else {
                continue;
            };
            if !action_ids.contains(action_id) {
                violations.push(violation(
                    "OPS_INVENTORY_GATE_ACTION_NOT_FOUND",
                    format!("gate action id `{action_id}` is not present in surfaces actions"),
                    "align ops/inventory/gates.json action_id fields with ops/inventory/surfaces.json",
                    Some(gates_rel),
                ));
            }
        }
    }

    let owners_text = fs::read_to_string(ctx.repo_root.join(owners_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let owners_json: serde_json::Value =
        serde_json::from_str(&owners_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let owner_areas = owners_json
        .get("areas")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let required_owner_prefixes = [
        "ops/datasets",
        "ops/e2e",
        "ops/env",
        "ops/inventory",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/schema",
        "ops/stack",
    ];
    for prefix in required_owner_prefixes {
        if !owner_areas.contains_key(prefix) {
            violations.push(violation(
                "OPS_OWNER_DOMAIN_MAPPING_MISSING",
                format!("owners.json is missing required area mapping `{prefix}`"),
                "add owner mapping for each top-level ops domain",
                Some(owners_rel),
            ));
        }
    }
    let owner_prefixes = owner_areas.keys().cloned().collect::<Vec<_>>();
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let rel_str = rel.display().to_string();
        if rel_str.starts_with("ops/_generated/") {
            continue;
        }
        let has_owner = owner_prefixes.iter().any(|prefix| {
            rel_str == *prefix
                || rel_str
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        });
        if !has_owner {
            violations.push(violation(
                "OPS_OWNER_ASSIGNMENT_MISSING",
                format!("ops file has no owner mapping in owners.json: `{rel_str}`"),
                "add an owners.json area mapping that covers this file path",
                Some(owners_rel),
            ));
        }
    }

    let registry_text = fs::read_to_string(ctx.repo_root.join(registry_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let tools_text = fs::read_to_string(ctx.repo_root.join(tools_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if registry_text.contains("[[tools]]") {
        violations.push(violation(
            "OPS_INVENTORY_REGISTRY_TOOLS_SURFACE_COLLISION",
            "ops/inventory/registry.toml must not define [[tools]] entries".to_string(),
            "keep registry.toml for checks/actions and tools.toml for tool probes",
            Some(registry_rel),
        ));
    }
    if tools_text.contains("[[checks]]") || tools_text.contains("[[actions]]") {
        violations.push(violation(
            "OPS_INVENTORY_TOOLS_REGISTRY_SURFACE_COLLISION",
            "ops/inventory/tools.toml must not define [[checks]] or [[actions]] entries"
                .to_string(),
            "keep tools.toml limited to [[tools]] entries",
            Some(tools_rel),
        ));
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, policy_schema_rel) {
        violations.push(violation(
            "OPS_INVENTORY_POLICY_SCHEMA_MISSING",
            format!("missing policy schema `{}`", policy_schema_rel.display()),
            "restore dev-atlas policy schema file",
            Some(policy_schema_rel),
        ));
    }
    let policy_text = fs::read_to_string(ctx.repo_root.join(policy_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let policy_json: serde_json::Value =
        serde_json::from_str(&policy_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    if policy_json.get("schema_version").is_none() || policy_json.get("mode").is_none() {
        violations.push(violation(
            "OPS_INVENTORY_POLICY_REQUIRED_KEYS_MISSING",
            "dev-atlas policy is missing required top-level keys".to_string(),
            "ensure dev-atlas policy includes at least schema_version and mode",
            Some(policy_rel),
        ));
    }

    let pins_text = fs::read_to_string(ctx.repo_root.join(pins_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let pins_yaml: serde_yaml::Value =
        serde_yaml::from_str(&pins_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let pins_images = pins_yaml
        .get("images")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();
    let stack_manifest_text = fs::read_to_string(ctx.repo_root.join(stack_manifest_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let stack_manifest_json: serde_json::Value = serde_json::from_str(&stack_manifest_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    if let Some(obj) = stack_manifest_json.as_object() {
        for (key, value) in obj {
            if key == "schema_version" {
                continue;
            }
            let image_value = value.as_str().unwrap_or_default();
            let pin_value = pins_images
                .get(serde_yaml::Value::String(key.clone()))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if pin_value != image_value {
                violations.push(violation(
                    "OPS_INVENTORY_PIN_STACK_DRIFT",
                    format!("stack manifest image `{key}` differs from inventory pin value"),
                    "regenerate stack generated version-manifest from inventory pins",
                    Some(stack_manifest_rel),
                ));
            }
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, inventory_index_rel) {
        violations.push(violation(
            "OPS_INVENTORY_INDEX_ARTIFACT_MISSING",
            format!(
                "missing generated inventory index artifact `{}`",
                inventory_index_rel.display()
            ),
            "generate and commit ops/_generated.example/inventory-index.json",
            Some(inventory_index_rel),
        ));
    } else {
        let inventory_index_text = fs::read_to_string(ctx.repo_root.join(inventory_index_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let inventory_index_json: serde_json::Value =
            serde_json::from_str(&inventory_index_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let indexed_paths = inventory_index_json
            .get("items")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("path").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect::<std::collections::BTreeSet<_>>()
            })
            .unwrap_or_default();
        let expected_inventory_paths = walk_files(&inventory_root)
            .into_iter()
            .filter_map(|file| file.strip_prefix(ctx.repo_root).ok().map(PathBuf::from))
            .filter(|rel| {
                rel.extension()
                    .and_then(|v| v.to_str())
                    .is_some_and(|ext| matches!(ext, "json" | "yaml" | "yml" | "toml"))
            })
            .map(|rel| rel.display().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        for rel in expected_inventory_paths.difference(&indexed_paths) {
            violations.push(violation(
                "OPS_INVENTORY_INDEX_COVERAGE_MISSING",
                format!("inventory-index.json is missing inventory artifact `{rel}`"),
                "regenerate ops/_generated.example/inventory-index.json to include every ops/inventory data artifact",
                Some(inventory_index_rel),
            ));
        }
    }

    if ctx.adapters.fs.exists(ctx.repo_root, stack_toml_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, stack_dependency_graph_rel)
    {
        let stack_toml_text = fs::read_to_string(ctx.repo_root.join(stack_toml_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let stack_toml: toml::Value =
            toml::from_str(&stack_toml_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let graph_text = fs::read_to_string(ctx.repo_root.join(stack_dependency_graph_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let graph_json: serde_json::Value =
            serde_json::from_str(&graph_text).map_err(|err| CheckError::Failed(err.to_string()))?;
        let toml_profiles = stack_toml
            .get("profiles")
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();
        let graph_profiles = graph_json
            .get("profiles")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let toml_profile_keys = toml_profiles.keys().cloned().collect::<BTreeSet<_>>();
        let graph_profile_keys = graph_profiles.keys().cloned().collect::<BTreeSet<_>>();
        if toml_profile_keys != graph_profile_keys {
            violations.push(violation(
                "OPS_STACK_DEPENDENCY_GRAPH_PROFILE_DRIFT",
                format!(
                    "dependency-graph profile keys drift from stack.toml: stack={toml_profile_keys:?} graph={graph_profile_keys:?}"
                ),
                "regenerate ops/stack/generated/dependency-graph.json from ops/stack/stack.toml",
                Some(stack_dependency_graph_rel),
            ));
        }

        if ctx.adapters.fs.exists(ctx.repo_root, stack_service_contract_rel) {
            let service_contract_text =
                fs::read_to_string(ctx.repo_root.join(stack_service_contract_rel))
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
            let service_contract_json: serde_json::Value =
                serde_json::from_str(&service_contract_text)
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
            let services = service_contract_json
                .get("services")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut has_critical_service = false;
            for service in services {
                let service_id = service
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown-service");
                let component = service
                    .get("component")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if component.is_empty() {
                    continue;
                }
                if service
                    .get("critical")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    has_critical_service = true;
                }
                let component_rel = Path::new(component);
                if !ctx.adapters.fs.exists(ctx.repo_root, component_rel) {
                    violations.push(violation(
                        "OPS_STACK_SERVICE_COMPONENT_MISSING",
                        format!(
                            "stack service `{service_id}` references missing component `{component}`"
                        ),
                        "fix component path in ops/stack/service-dependency-contract.json",
                        Some(stack_service_contract_rel),
                    ));
                    continue;
                }
                for profile in service
                    .get("required_profiles")
                    .and_then(|v| v.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str())
                {
                    let Some(profile_value) = toml_profiles.get(profile) else {
                        violations.push(violation(
                            "OPS_STACK_SERVICE_REQUIRED_PROFILE_MISSING",
                            format!(
                                "stack service `{service_id}` requires unknown profile `{profile}`"
                            ),
                            "align required_profiles with ops/stack/stack.toml profile keys",
                            Some(stack_service_contract_rel),
                        ));
                        continue;
                    };
                    let profile_components = profile_value
                        .get("components")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();
                    let profile_component_paths = profile_components
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<BTreeSet<_>>();
                    if !profile_component_paths.contains(component) {
                        violations.push(violation(
                            "OPS_STACK_SERVICE_PROFILE_COMPONENT_DRIFT",
                            format!(
                                "stack service `{service_id}` component `{component}` missing from profile `{profile}` components"
                            ),
                            "keep service-dependency-contract required_profiles aligned with stack.toml components",
                            Some(stack_service_contract_rel),
                        ));
                    }
                }
            }
            if !has_critical_service {
                violations.push(violation(
                    "OPS_STACK_SERVICE_CRITICAL_COVERAGE_MISSING",
                    "service-dependency-contract must define at least one critical service"
                        .to_string(),
                    "mark core stack services as critical in ops/stack/service-dependency-contract.json",
                    Some(stack_service_contract_rel),
                ));
            }
        } else {
            violations.push(violation(
                "OPS_STACK_SERVICE_CONTRACT_MISSING",
                format!(
                    "missing stack service dependency contract `{}`",
                    stack_service_contract_rel.display()
                ),
                "restore ops/stack/service-dependency-contract.json",
                Some(stack_service_contract_rel),
            ));
        }
    }

    if ctx.adapters.fs.exists(ctx.repo_root, stack_evolution_policy_rel) {
        let evolution_text = fs::read_to_string(ctx.repo_root.join(stack_evolution_policy_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let evolution_json: serde_json::Value = serde_json::from_str(&evolution_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in [
            "schema_version",
            "policy_version",
            "freeze_rules",
            "change_controls",
            "compatibility",
        ] {
            if evolution_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_STACK_EVOLUTION_POLICY_FIELD_MISSING",
                    format!("stack evolution policy missing required key `{key}`"),
                    "add missing required keys to ops/stack/evolution-policy.json",
                    Some(stack_evolution_policy_rel),
                ));
            }
        }
    } else {
        violations.push(violation(
            "OPS_STACK_EVOLUTION_POLICY_MISSING",
            format!(
                "missing stack evolution policy `{}`",
                stack_evolution_policy_rel.display()
            ),
            "restore ops/stack/evolution-policy.json",
            Some(stack_evolution_policy_rel),
        ));
    }

    if ctx.adapters.fs.exists(ctx.repo_root, k8s_install_matrix_rel)
        && ctx.adapters.fs.exists(ctx.repo_root, k8s_rollout_contract_rel)
    {
        let install_matrix_text = fs::read_to_string(ctx.repo_root.join(k8s_install_matrix_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let install_matrix_json: serde_json::Value = serde_json::from_str(&install_matrix_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let matrix_profiles = install_matrix_json
            .get("profiles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let matrix_by_name = matrix_profiles
            .iter()
            .filter_map(|entry| {
                let name = entry.get("name").and_then(|v| v.as_str())?;
                Some((name.to_string(), entry.clone()))
            })
            .collect::<BTreeMap<_, _>>();

        let rollout_contract_text =
            fs::read_to_string(ctx.repo_root.join(k8s_rollout_contract_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
        let rollout_contract_json: serde_json::Value =
            serde_json::from_str(&rollout_contract_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
        let rollout_profiles = rollout_contract_json
            .get("profiles")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for profile in &rollout_profiles {
            let profile_name = profile
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if profile_name.is_empty() {
                continue;
            }
            let Some(matrix_entry) = matrix_by_name.get(profile_name) else {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_PROFILE_MISSING_FROM_INSTALL_MATRIX",
                    format!(
                        "rollout-safety-contract profile `{profile_name}` is missing from install-matrix"
                    ),
                    "align rollout-safety-contract profiles with ops/k8s/install-matrix.json",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            };
            let contract_suite = profile.get("suite").and_then(|v| v.as_str()).unwrap_or_default();
            let matrix_suite = matrix_entry
                .get("suite")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if contract_suite != matrix_suite {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_SUITE_DRIFT",
                    format!(
                        "rollout-safety-contract suite drift for profile `{profile_name}`: contract=`{contract_suite}` matrix=`{matrix_suite}`"
                    ),
                    "align rollout-safety-contract suite values with install-matrix",
                    Some(k8s_rollout_contract_rel),
                ));
            }
            let values_file = profile
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let matrix_values_file = matrix_entry
                .get("values_file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if values_file != matrix_values_file {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_VALUES_FILE_DRIFT",
                    format!(
                        "rollout-safety-contract values_file drift for profile `{profile_name}`: contract=`{values_file}` matrix=`{matrix_values_file}`"
                    ),
                    "align rollout-safety-contract values_file with install-matrix",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            }
            let values_rel = Path::new(values_file);
            if !ctx.adapters.fs.exists(ctx.repo_root, values_rel) {
                violations.push(violation(
                    "OPS_K8S_ROLLOUT_VALUES_FILE_MISSING",
                    format!("rollout-safety-contract references missing values file `{values_file}`"),
                    "restore missing values file or update rollout-safety-contract",
                    Some(k8s_rollout_contract_rel),
                ));
                continue;
            }
            let values_text = fs::read_to_string(ctx.repo_root.join(values_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let values_yaml: serde_yaml::Value = serde_yaml::from_str(&values_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let warmup_required = profile
                .get("warmup_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if warmup_required {
                let warmup_enabled = values_yaml
                    .get("cache")
                    .and_then(|v| v.get("warmupEnabled"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !warmup_enabled {
                    violations.push(violation(
                        "OPS_K8S_WARMUP_REQUIRED_BUT_DISABLED",
                        format!(
                            "profile `{profile_name}` requires warmup but values file disables cache.warmupEnabled"
                        ),
                        "enable cache.warmupEnabled for warmup-required rollout profiles",
                        Some(values_rel),
                    ));
                }
            }
            let readiness_required = profile
                .get("readiness_path_required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if readiness_required {
                let readiness_path = values_yaml
                    .get("server")
                    .and_then(|v| v.get("readinessProbePath"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if readiness_path.trim().is_empty() {
                    violations.push(violation(
                        "OPS_K8S_READINESS_PATH_REQUIRED_MISSING",
                        format!(
                            "profile `{profile_name}` requires readiness probe path but server.readinessProbePath is missing"
                        ),
                        "define server.readinessProbePath in profile values file",
                        Some(values_rel),
                    ));
                }
            }
        }
    } else if !ctx.adapters.fs.exists(ctx.repo_root, k8s_rollout_contract_rel) {
        violations.push(violation(
            "OPS_K8S_ROLLOUT_CONTRACT_MISSING",
            format!(
                "missing k8s rollout safety contract `{}`",
                k8s_rollout_contract_rel.display()
            ),
            "restore ops/k8s/rollout-safety-contract.json",
            Some(k8s_rollout_contract_rel),
        ));
    }

    let registry_drift_rel = Path::new("ops/_generated.example/registry-drift-report.json");
    let control_graph_diff_rel = Path::new("ops/_generated.example/control-graph-diff-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, registry_drift_rel) {
        violations.push(violation(
            "OPS_INVENTORY_REGISTRY_DRIFT_REPORT_MISSING",
            format!(
                "missing registry drift report `{}`",
                registry_drift_rel.display()
            ),
            "generate and commit ops/_generated.example/registry-drift-report.json",
            Some(registry_drift_rel),
        ));
    } else {
        let registry_drift_text = fs::read_to_string(ctx.repo_root.join(registry_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let registry_drift_json: serde_json::Value = serde_json::from_str(&registry_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if registry_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_INVENTORY_REGISTRY_DRIFT_REPORT_BLOCKING",
                "registry-drift-report.json status is not `pass`".to_string(),
                "resolve inventory registry drift and regenerate registry-drift-report.json",
                Some(registry_drift_rel),
            ));
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, control_graph_diff_rel) {
        violations.push(violation(
            "OPS_INVENTORY_CONTROL_GRAPH_DIFF_REPORT_MISSING",
            format!(
                "missing control graph diff report `{}`",
                control_graph_diff_rel.display()
            ),
            "generate and commit ops/_generated.example/control-graph-diff-report.json",
            Some(control_graph_diff_rel),
        ));
    } else {
        let control_graph_diff_text = fs::read_to_string(ctx.repo_root.join(control_graph_diff_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let control_graph_diff_json: serde_json::Value =
            serde_json::from_str(&control_graph_diff_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
        if control_graph_diff_json
            .get("status")
            .and_then(|v| v.as_str())
            != Some("pass")
        {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_DIFF_REPORT_BLOCKING",
                "control-graph-diff-report.json status is not `pass`".to_string(),
                "resolve control graph drift and regenerate control-graph-diff-report.json",
                Some(control_graph_diff_rel),
            ));
        }
    }

    if !ctx.adapters.fs.exists(ctx.repo_root, stack_drift_rel) {
        violations.push(violation(
            "OPS_STACK_DRIFT_REPORT_MISSING",
            format!("missing stack drift report `{}`", stack_drift_rel.display()),
            "generate and commit ops/_generated.example/stack-drift-report.json",
            Some(stack_drift_rel),
        ));
    } else {
        let stack_drift_text = fs::read_to_string(ctx.repo_root.join(stack_drift_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let stack_drift_json: serde_json::Value = serde_json::from_str(&stack_drift_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if stack_drift_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
            violations.push(violation(
                "OPS_STACK_DRIFT_REPORT_BLOCKING",
                "stack-drift-report.json status is not `pass`".to_string(),
                "resolve stack drift and regenerate stack-drift-report.json",
                Some(stack_drift_rel),
            ));
        }
    }

    Ok(violations)
}

