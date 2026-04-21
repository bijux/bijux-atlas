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
    #[serde(rename = "class")]
    telemetry_class: String,
}

#[test]
fn server_routes_match_endpoints_contract_and_telemetry_annotations() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let contract_path = root.join("ops/observe/contracts/endpoint-observability-contract.json");
    let contract: EndpointsContract =
        serde_json::from_slice(&std::fs::read(contract_path).expect("read endpoints contract"))
            .expect("parse endpoints contract");

    let server_src = std::fs::read_to_string(
        root.join("crates/bijux-atlas/src/adapters/inbound/http/router.rs"),
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
        let method = ep.method.to_ascii_uppercase();
        assert!(
            matches!(method.as_str(), "GET" | "POST"),
            "unsupported method in v1 route registry: {} {}",
            ep.method,
            ep.path
        );
        assert!(
            !ep.telemetry_class.trim().is_empty(),
            "missing telemetry_class for {} {}",
            method,
            ep.path
        );
        contract_set.insert(ep.path.clone());
    }

    let missing_from_server = contract_set
        .difference(&route_set)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing_from_server.is_empty(),
        "server route registry drift; missing documented routes: {:?}",
        missing_from_server
    );
}

#[test]
fn openapi_path_params_match_path_templates() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();
    let openapi: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("configs/generated/openapi/v1/openapi.json"))
            .expect("read openapi"),
    )
    .expect("parse openapi");
    let path_param_re = regex::Regex::new(r"\{([^}]+)\}").expect("path param regex");
    for (path, methods) in openapi["paths"].as_object().expect("paths object") {
        let expected = path_param_re
            .captures_iter(path)
            .filter_map(|capture| {
                capture
                    .get(1)
                    .map(|m| ("path".to_string(), m.as_str().to_string()))
            })
            .collect::<std::collections::BTreeSet<_>>();
        for method in ["get", "post"] {
            if methods.get(method).is_none() {
                continue;
            }
            let actual = methods[method]["parameters"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| {
                    let name = value.get("name")?.as_str()?.to_string();
                    let in_ = value.get("in")?.as_str()?.to_string();
                    Some((in_, name))
                })
                .filter(|(in_, _)| in_ == "path")
                .collect::<std::collections::BTreeSet<_>>();
            if !actual.is_empty() || !expected.is_empty() {
                assert_eq!(
                    expected, actual,
                    "path param registry drift for {} {}",
                    method, path
                );
            }
        }
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
        &std::fs::read(root.join("configs/generated/openapi/v1/openapi.json"))
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
    let docs =
        std::fs::read_to_string(root.join("docs/bijux-atlas/interfaces/api-endpoint-index.md"))
            .expect("read docs");
    assert!(
        docs.contains("/v1/datasets/{release}/{species}/{assembly}"),
        "canonical dataset path missing from V1 surface docs"
    );
}
