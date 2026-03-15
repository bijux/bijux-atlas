// SPDX-License-Identifier: Apache-2.0

use crate::*;
use brotli::CompressorWriter;
use flate2::{Compression, write::GzEncoder};
use serde_json::{Value, json};
use std::io::Write;

pub(crate) fn is_gene_id_exact_query(req: &GeneQueryRequest) -> Option<&str> {
    let gene_id = req.filter.gene_id.as_deref()?;
    if req.filter.name.is_none()
        && req.filter.name_prefix.is_none()
        && req.filter.biotype.is_none()
        && req.filter.region.is_none()
        && req.cursor.is_none()
        && req.limit <= 1
    {
        Some(gene_id)
    } else {
        None
    }
}

pub(crate) fn gene_fields_key(fields: &GeneFields) -> String {
    format!(
        "{}{}{}{}{}{}",
        fields.gene_id as u8,
        fields.name as u8,
        fields.coords as u8,
        fields.biotype as u8,
        fields.transcript_count as u8,
        fields.sequence_length as u8
    )
}

pub(crate) fn accepted_encoding(headers: &HeaderMap) -> Option<&'static str> {
    let accept = headers
        .get("accept-encoding")
        .and_then(|v| v.to_str().ok())?;
    if accept.contains("br") {
        Some("br")
    } else if accept.contains("gzip") {
        Some("gzip")
    } else {
        None
    }
}

pub(crate) fn serialize_payload_with_capacity(
    payload: &Value,
    pretty: bool,
    capacity_hint: usize,
) -> Result<Vec<u8>, ApiError> {
    let mut out = Vec::with_capacity(capacity_hint);
    if pretty {
        serde_json::to_writer_pretty(&mut out, payload).map_err(|e| {
            crate::interfaces::http::presenters::error_json(
                ApiErrorCode::Internal,
                "json serialization failed",
                json!({"message": e.to_string()}),
            )
        })?;
    } else {
        serde_json::to_writer(&mut out, payload).map_err(|e| {
            crate::interfaces::http::presenters::error_json(
                ApiErrorCode::Internal,
                "json serialization failed",
                json!({"message": e.to_string()}),
            )
        })?;
    }
    Ok(out)
}

pub(crate) fn maybe_compress_response(
    headers: &HeaderMap,
    state: &AppState,
    bytes: Vec<u8>,
) -> Result<(Vec<u8>, Option<&'static str>), ApiError> {
    if !state.api.enable_response_compression || bytes.len() < state.api.compression_min_bytes {
        return Ok((bytes, None));
    }
    match accepted_encoding(headers) {
        Some("gzip") => {
            let mut encoder = GzEncoder::new(
                Vec::with_capacity((bytes.len() / 2).max(256)),
                Compression::fast(),
            );
            encoder.write_all(&bytes).map_err(|e| {
                crate::interfaces::http::presenters::error_json(
                    ApiErrorCode::Internal,
                    "gzip encoding failed",
                    json!({"message": e.to_string()}),
                )
            })?;
            let compressed = encoder.finish().map_err(|e| {
                crate::interfaces::http::presenters::error_json(
                    ApiErrorCode::Internal,
                    "gzip finalize failed",
                    json!({"message": e.to_string()}),
                )
            })?;
            Ok((compressed, Some("gzip")))
        }
        Some("br") => {
            let mut compressed = Vec::with_capacity((bytes.len() / 2).max(256));
            {
                let mut writer = CompressorWriter::new(&mut compressed, 4096, 4, 22);
                writer.write_all(&bytes).map_err(|e| {
                    crate::interfaces::http::presenters::error_json(
                        ApiErrorCode::Internal,
                        "brotli encoding failed",
                        json!({"message": e.to_string()}),
                    )
                })?;
            }
            Ok((compressed, Some("br")))
        }
        _ => Ok((bytes, None)),
    }
}
