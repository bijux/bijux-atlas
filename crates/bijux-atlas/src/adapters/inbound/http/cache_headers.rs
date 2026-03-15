// SPDX-License-Identifier: Apache-2.0

use crate::*;
use std::collections::HashMap;

pub(crate) fn normalize_query(params: &HashMap<String, String>) -> String {
    let mut kv: Vec<(&String, &String)> = params.iter().collect();
    kv.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(b.1)));
    kv.into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

pub(crate) fn if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get("if-none-match")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string)
}

pub(crate) enum CachePolicy {
    ImmutableDataset,
    CatalogDiscovery,
}

pub(crate) fn put_cache_headers(
    headers: &mut HeaderMap,
    ttl: Duration,
    etag: &str,
    policy: CachePolicy,
) {
    let stale = (ttl.as_secs() / 2).max(1);
    let cache_control = match policy {
        CachePolicy::ImmutableDataset => {
            format!(
                "public, max-age={}, stale-while-revalidate={}, immutable",
                ttl.as_secs(),
                stale
            )
        }
        CachePolicy::CatalogDiscovery => format!(
            "public, max-age={}, stale-while-revalidate={}",
            ttl.as_secs(),
            stale
        ),
    };
    if let Ok(value) = HeaderValue::from_str(&cache_control) {
        headers.insert("cache-control", value);
    }
    if let Ok(value) = HeaderValue::from_str(etag) {
        headers.insert("etag", value);
    }
    headers.insert("vary", HeaderValue::from_static("accept-encoding"));
}

pub(crate) fn dataset_artifact_hash(
    manifest: Option<&ArtifactManifest>,
    dataset: &DatasetId,
) -> String {
    if let Some(summary) = manifest {
        if !summary.dataset_signature_sha256.trim().is_empty() {
            return summary.dataset_signature_sha256.clone();
        }
    }
    sha256_hex(dataset.canonical_string().as_bytes())
}

pub(crate) fn dataset_etag(
    artifact_hash: &str,
    path: &str,
    params: &HashMap<String, String>,
) -> String {
    let normalized = normalize_query(params);
    format!(
        "\"{}\"",
        sha256_hex(format!("{artifact_hash}|{path}|{normalized}").as_bytes())
    )
}

pub(crate) fn cache_debug_headers(
    headers: &mut HeaderMap,
    enabled: bool,
    artifact_hash: &str,
    normalized_request: &str,
) {
    if !enabled {
        return;
    }
    if let Ok(v) = HeaderValue::from_str(artifact_hash) {
        headers.insert("x-atlas-artifact-hash", v);
    }
    if let Ok(v) = HeaderValue::from_str(normalized_request) {
        headers.insert("x-atlas-cache-key", v);
    }
}

pub(crate) fn wants_pretty(params: &HashMap<String, String>) -> bool {
    params
        .get("pretty")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

pub(crate) fn wants_min_viable_response(params: &HashMap<String, String>) -> bool {
    params
        .get("mvr")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

pub(crate) fn wants_text(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/plain"))
}

pub(crate) fn bool_query_flag(params: &HashMap<String, String>, name: &str) -> bool {
    params
        .get(name)
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}
