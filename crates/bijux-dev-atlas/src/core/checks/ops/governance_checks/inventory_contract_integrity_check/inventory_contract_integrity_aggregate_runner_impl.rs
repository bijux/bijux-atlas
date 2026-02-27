fn check_ops_inventory_contract_integrity_aggregate_runner(
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
    let mut node_paths = std::collections::BTreeMap::<String, String>::new();
    let mut path_prefix_coverage = std::collections::BTreeSet::<&'static str>::new();
    let valid_node_lifecycles = std::collections::BTreeSet::from([
        "authoritative",
        "generated_committed",
        "generated_example",
        "runtime_generated",
        "declared",
    ]);
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
        let node_consumer = node
            .get("consumer")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim();
        if node_consumer.is_empty() {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_NODE_CONSUMER_MISSING",
                format!("control graph node `{id}` is missing non-empty `consumer` metadata"),
                "add `consumer` to every control-graph node",
                Some(control_graph_rel),
            ));
        }
        let node_lifecycle = node
            .get("lifecycle")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim();
        if !valid_node_lifecycles.contains(node_lifecycle) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_NODE_LIFECYCLE_INVALID",
                format!(
                    "control graph node `{id}` has missing/invalid `lifecycle` metadata: `{node_lifecycle}`"
                ),
                "set lifecycle to one of authoritative/generated_committed/generated_example/runtime_generated/declared",
                Some(control_graph_rel),
            ));
        }
        if node_lifecycle.starts_with("generated_")
            && node
                .get("producer")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_GENERATED_NODE_PRODUCER_MISSING",
                format!("generated control graph node `{id}` is missing non-empty `producer` metadata"),
                "add `producer` command metadata to generated control-graph nodes",
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
            if let Some(prev_id) = node_paths.insert(path.to_string(), id.to_string()) {
                violations.push(violation(
                    "OPS_INVENTORY_CONTROL_GRAPH_DUPLICATE_PATH_NODE",
                    format!(
                        "control graph path `{path}` is declared by multiple nodes: `{prev_id}` and `{id}`"
                    ),
                    "assign one control graph node per concrete path",
                    Some(control_graph_rel),
                ));
            }
            for (prefix, key) in [
                ("docs/", "docs"),
                ("makefiles/", "make"),
                ("crates/", "runtime"),
                ("ops/report/", "report"),
                ("ops/schema/", "schema"),
            ] {
                if path.starts_with(prefix) {
                    path_prefix_coverage.insert(key);
                }
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
    let mut incident_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut covered_domain_nodes = std::collections::BTreeSet::<String>::new();
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
        *incident_counts.entry(from.to_string()).or_default() += 1;
        *incident_counts.entry(to.to_string()).or_default() += 1;
        if kind == "coverage" && to.starts_with("domain.") {
            covered_domain_nodes.insert(to.to_string());
        }
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
    for node_id in &node_ids {
        if !incident_counts.contains_key(node_id) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_ORPHAN_NODE",
                format!("control graph node `{node_id}` has no incident edges"),
                "connect every control graph node with at least one incoming or outgoing edge",
                Some(control_graph_rel),
            ));
        }
    }
    for required_kind in [
        "dependency",
        "consumer",
        "producer",
        "lifecycle",
        "drift",
        "schema_link",
    ] {
        if !seen_kinds.contains(required_kind) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_EDGE_KIND_MISSING",
                format!("control graph is missing required edge kind `{required_kind}`"),
                "add at least one edge for each required control graph edge kind",
                Some(control_graph_rel),
            ));
        }
    }
    for required_domain in [
        "domain.datasets",
        "domain.e2e",
        "domain.env",
        "domain.inventory",
        "domain.k8s",
        "domain.load",
        "domain.observe",
        "domain.report",
        "domain.schema",
        "domain.stack",
    ] {
        if !node_ids.contains(required_domain) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_DOMAIN_NODE_MISSING",
                format!("control graph is missing canonical domain node `{required_domain}`"),
                "add canonical ops domain nodes to control-graph.json for coverage accounting",
                Some(control_graph_rel),
            ));
            continue;
        }
        if !covered_domain_nodes.contains(required_domain) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_DOMAIN_COVERAGE_MISSING",
                format!("control graph domain node `{required_domain}` has no coverage edge"),
                "add a coverage edge to every canonical domain node",
                Some(control_graph_rel),
            ));
        }
    }
    for required_prefix in ["docs", "make", "runtime", "report", "schema"] {
        if !path_prefix_coverage.contains(required_prefix) {
            violations.push(violation(
                "OPS_INVENTORY_CONTROL_GRAPH_PATH_COVERAGE_MISSING",
                format!(
                    "control graph is missing explicit `{required_prefix}` path-backed node coverage"
                ),
                "add control graph nodes for docs, makefiles, runtime crate, schema, and report artifact paths",
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

    validate_inventory_owners_registry_and_policy_contracts(
        ctx,
        &mut violations,
        owners_rel,
        registry_rel,
        tools_rel,
        policy_rel,
        policy_schema_rel,
    )?;

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

    validate_generated_inventory_index_and_stack_contracts(
        ctx,
        &mut violations,
        InventoryAndStackContractPaths {
            inventory_root: &inventory_root,
            inventory_index_rel,
            stack_toml_rel,
            stack_dependency_graph_rel,
            stack_service_contract_rel,
            stack_evolution_policy_rel,
        },
    )?;
    validate_k8s_rollout_and_drift_reports(
        ctx,
        &mut violations,
        k8s_install_matrix_rel,
        k8s_rollout_contract_rel,
        stack_drift_rel,
    )?;

    Ok(violations)
}
