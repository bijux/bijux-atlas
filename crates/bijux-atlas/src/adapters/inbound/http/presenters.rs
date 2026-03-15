// SPDX-License-Identifier: Apache-2.0

use crate::*;
use serde_json::{Value, json};
use std::sync::atomic::Ordering;

pub(crate) use crate::http::response_contract::api_error as error_json;
pub(crate) use crate::http::response_contract::api_error_response;

pub(crate) fn json_envelope(
    dataset: Option<Value>,
    page: Option<Value>,
    data: Value,
    links: Option<Value>,
    warnings: Option<Vec<Value>>,
) -> Value {
    let mut root = json!({
        "api_version": "v1",
        "contract_version": "v1",
        "dataset": dataset.unwrap_or(Value::Null),
        "page": page.unwrap_or(Value::Null),
        "data": data,
        "links": links.unwrap_or(Value::Null)
    });
    if let Some(warnings) = warnings {
        if !warnings.is_empty() {
            root["meta"] = json!({ "warnings": warnings });
        }
    }
    root
}

pub(crate) fn with_request_id(mut response: Response, request_id: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", v);
    }
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-trace-id", v);
    }
    let should_set_json = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_none_or(|v| v.eq_ignore_ascii_case("application/json"));
    if should_set_json
        && !matches!(
            response.status(),
            StatusCode::NO_CONTENT | StatusCode::NOT_MODIFIED
        )
    {
        response.headers_mut().insert(
            "content-type",
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
    }
    response
}

pub(crate) fn with_query_class(mut response: Response, class: QueryClass) -> Response {
    let value = match class {
        QueryClass::Cheap => "cheap",
        QueryClass::Medium => "medium",
        QueryClass::Heavy => "heavy",
    };
    response
        .headers_mut()
        .insert("x-atlas-query-class", HeaderValue::from_static(value));
    response
}

pub(crate) async fn dataset_provenance(state: &AppState, dataset: &DatasetId) -> Value {
    let dataset_hash = sha256_hex(dataset.canonical_string().as_bytes());
    let mut out = json!({
        "dataset_hash": dataset_hash,
        "release": dataset.release,
        "species": dataset.species,
        "assembly": dataset.assembly
    });
    if let Ok(manifest) = state.cache.fetch_manifest_summary(dataset).await {
        out["manifest_version"] = json!(manifest.manifest_version);
        out["db_schema_version"] = json!(manifest.db_schema_version);
        out["dataset_signature_sha256"] = json!(manifest.dataset_signature_sha256);
    }
    out
}

pub(crate) fn is_draining(state: &AppState) -> bool {
    !state.accepting_requests.load(Ordering::Relaxed)
}
