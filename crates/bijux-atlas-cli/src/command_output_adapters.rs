use crate::OutputMode;
use bijux_atlas_core::canonical;
use bijux_atlas_query::{GeneFields, GeneFilter, GeneQueryRequest, RegionFilter};
use serde_json::{json, Value};
use std::path::PathBuf;

pub(crate) fn run_openapi_generate(out: PathBuf, output_mode: OutputMode) -> Result<(), String> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "failed to resolve workspace root".to_string())?
        .to_path_buf();
    let openapi_source = workspace_root.join("configs/openapi/v1/openapi.generated.json");
    let raw = std::fs::read(&openapi_source)
        .map_err(|e| format!("failed to read {}: {e}", openapi_source.display()))?;
    let spec: Value =
        serde_json::from_slice(&raw).map_err(|e| format!("invalid OpenAPI JSON: {e}"))?;
    let bytes = canonical::stable_json_bytes(&spec).map_err(|e| e.to_string())?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&out, bytes).map_err(|e| e.to_string())?;
    emit_ok(
        output_mode,
        json!({
            "command":"atlas openapi generate",
            "status":"ok",
            "out": out
        }),
    )?;
    Ok(())
}

pub(crate) fn emit_ok(output_mode: OutputMode, payload: Value) -> Result<(), String> {
    if output_mode.json {
        let bytes = canonical::stable_json_bytes(&payload).map_err(|e| e.to_string())?;
        let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        println!("{text}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

pub(crate) fn parse_dataset_id(dataset: &str) -> Result<(String, String, String), String> {
    let parts: Vec<&str> = dataset.split('/').collect();
    if parts.len() != 3 {
        return Err("dataset must be release/species/assembly".to_string());
    }
    Ok((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

pub(crate) fn query_request_from_json(v: &Value) -> Result<GeneQueryRequest, String> {
    let gene_id = v
        .get("gene_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let name = v
        .get("name")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let name_prefix = v
        .get("name_prefix")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let biotype = v
        .get("biotype")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let region = if let Some(raw) = v.get("region").and_then(Value::as_str) {
        let (seqid, span) = raw
            .split_once(':')
            .ok_or_else(|| "region must be seqid:start-end".to_string())?;
        let (start, end) = span
            .split_once('-')
            .ok_or_else(|| "region must be seqid:start-end".to_string())?;
        Some(RegionFilter {
            seqid: seqid.to_string(),
            start: start.parse::<u64>().map_err(|e| e.to_string())?,
            end: end.parse::<u64>().map_err(|e| e.to_string())?,
        })
    } else {
        None
    };
    let limit = v.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
    let allow_full_scan = v
        .get("allow_full_scan")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id,
            name,
            name_prefix,
            biotype,
            region,
        },
        limit,
        cursor: None,
        dataset_key: None,
        allow_full_scan,
    })
}
