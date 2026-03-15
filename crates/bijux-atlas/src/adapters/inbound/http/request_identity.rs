// SPDX-License-Identifier: Apache-2.0

use crate::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) struct RequestQueueGuard {
    pub(crate) counter: Arc<AtomicU64>,
}

impl Drop for RequestQueueGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) fn make_request_id(state: &AppState) -> String {
    let id = state.request_id_seed.fetch_add(1, Ordering::Relaxed);
    format!("req-{id:016x}")
}

pub(crate) fn propagated_request_id(headers: &HeaderMap, state: &AppState) -> String {
    crate::adapters::inbound::http::request_tracing::extract_request_trace(headers, state).request_id
}

pub(crate) fn normalized_forwarded_for(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("x-forwarded-for")?.to_str().ok()?;
    let first = raw.split(',').next()?.trim();
    if first.is_empty() || first.len() > 64 {
        return None;
    }
    if first
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b':' || b == b'-')
    {
        Some(first.to_string())
    } else {
        None
    }
}

pub(crate) fn normalized_api_key(headers: &HeaderMap) -> Option<String> {
    let key = headers.get("x-api-key")?.to_str().ok()?.trim();
    if key.is_empty() || key.len() > 256 {
        return None;
    }
    Some(key.to_string())
}

pub(crate) fn parse_region_opt(raw: Option<String>) -> Option<RegionFilter> {
    let value = raw?;
    let (seqid, span) = value.split_once(':')?;
    let (start, end) = span.split_once('-')?;
    Some(RegionFilter {
        seqid: seqid.to_string(),
        start: start.parse::<u64>().ok()?,
        end: end.parse::<u64>().ok()?,
    })
}
