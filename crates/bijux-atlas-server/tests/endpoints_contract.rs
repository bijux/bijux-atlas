use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EndpointsContract {
    endpoints: Vec<EndpointEntry>,
}

#[derive(Debug, Deserialize)]
struct EndpointEntry {
    method: String,
    path: String,
    telemetry_class: String,
}

#[test]
fn server_routes_match_endpoints_contract_and_telemetry_annotations() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let contract_path = root.join("docs/contracts/ENDPOINTS.json");
    let contract: EndpointsContract =
        serde_json::from_slice(&std::fs::read(contract_path).expect("read endpoints contract"))
            .expect("parse endpoints contract");

    let server_src = std::fs::read_to_string(
        root.join("crates/bijux-atlas-server/src/runtime/server_runtime_app.rs"),
    )
    .expect("read server routing source");

    let mut route_set = std::collections::BTreeSet::new();
    let param_re = regex::Regex::new(r":([A-Za-z_][A-Za-z0-9_]*)").expect("param regex");
    for cap in regex::Regex::new(r#"\.route\(\s*"([^"]+)""#)
        .expect("regex")
        .captures_iter(&server_src)
    {
        let mut path = cap[1].to_string();
        path = param_re.replace_all(&path, "{$1}").to_string();
        if path != "/" {
            route_set.insert(path);
        }
    }

    let mut contract_set = std::collections::BTreeSet::new();
    for ep in &contract.endpoints {
        assert_eq!(
            ep.method, "GET",
            "only GET v1 routes are currently supported"
        );
        assert!(
            !ep.telemetry_class.trim().is_empty(),
            "missing telemetry_class for {} {}",
            ep.method,
            ep.path
        );
        contract_set.insert(ep.path.clone());
    }

    assert_eq!(route_set, contract_set, "server route registry drift");
}
