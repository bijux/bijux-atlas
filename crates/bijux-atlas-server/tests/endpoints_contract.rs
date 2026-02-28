// SPDX-License-Identifier: Apache-2.0

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
    #[serde(default)]
    params: Vec<EndpointParam>,
}

#[derive(Debug, Deserialize)]
struct EndpointParam {
    name: String,
    #[serde(rename = "in")]
    in_: String,
}

#[test]
fn server_routes_match_endpoints_contract_and_telemetry_annotations() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let contract_path = root.join("docs/reference/contracts/schemas/ENDPOINTS.json");
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
        assert!(
            matches!(ep.method.as_str(), "GET" | "POST"),
            "unsupported method in v1 route registry: {} {}",
            ep.method,
            ep.path
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

#[test]
fn endpoint_params_match_openapi_registry() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let contract: EndpointsContract = serde_json::from_slice(
        &std::fs::read(root.join("docs/reference/contracts/schemas/ENDPOINTS.json")).expect("read endpoints"),
    )
    .expect("parse endpoints");
    let openapi: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("configs/openapi/v1/openapi.generated.json"))
            .expect("read openapi"),
    )
    .expect("parse openapi");
    for ep in &contract.endpoints {
        let actual = openapi
            .pointer(&format!(
                "/paths/{}/get/parameters",
                ep.path.replace('/', "~1")
            ))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| {
                let name = v.get("name")?.as_str()?.to_string();
                let in_ = v.get("in")?.as_str()?.to_string();
                Some((in_, name))
            })
            .collect::<std::collections::BTreeSet<_>>();
        let expected = ep
            .params
            .iter()
            .map(|p| (p.in_.clone(), p.name.clone()))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(expected, actual, "param registry drift for {}", ep.path);
    }
}

#[test]
fn openapi_marks_legacy_dataset_path_deprecated_and_has_canonical_path() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let openapi: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("configs/openapi/v1/openapi.generated.json"))
            .expect("read openapi"),
    )
    .expect("parse openapi");

    assert!(openapi
        .pointer("/paths/~1v1~1datasets~1{release}~1{species}~1{assembly}/get")
        .is_some());
    assert_eq!(
        openapi
            .pointer("/paths/~1v1~1releases~1{release}~1species~1{species}~1assemblies~1{assembly}/get/deprecated")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

#[test]
fn docs_reference_canonical_dataset_path() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let docs = std::fs::read_to_string(root.join("docs/api/v1-surface.md")).expect("read docs");
    assert!(
        docs.contains("/v1/datasets/{release}/{species}/{assembly}"),
        "canonical dataset path missing from V1 surface docs"
    );
}
