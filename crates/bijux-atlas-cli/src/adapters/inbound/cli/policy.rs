// SPDX-License-Identifier: Apache-2.0

use super::*;
use bijux_atlas_runtime::domain::policy::{
    canonical_config_json, load_policy_from_workspace, resolve_mode_profile, PolicyMode,
};

pub(crate) fn validate_policy(output_mode: OutputMode) -> Result<(), String> {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "failed to resolve workspace root from CARGO_MANIFEST_DIR".to_string())?
        .to_path_buf();
    let policy = load_policy_from_workspace(&workspace).map_err(|e| e.to_string())?;
    let canonical = canonical_config_json(&policy).map_err(|e| e.to_string())?;
    if output_mode.json {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "command":"atlas policy validate",
                "status":"ok",
                "schema_version": policy.schema_version.as_str(),
                "canonical": serde_json::from_str::<serde_json::Value>(&canonical).map_err(|e| e.to_string())?
            }))
            .map_err(|e| e.to_string())?
        );
    } else {
        println!("{canonical}");
    }
    Ok(())
}

pub(crate) fn explain_policy(
    mode_override: Option<PolicyMode>,
    output_mode: OutputMode,
) -> Result<(), String> {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "failed to resolve workspace root from CARGO_MANIFEST_DIR".to_string())?
        .to_path_buf();
    let policy = load_policy_from_workspace(&workspace).map_err(|e| e.to_string())?;
    let active_mode = mode_override.unwrap_or(policy.mode);
    let strict = resolve_mode_profile(&policy, PolicyMode::Strict).map_err(|e| e.to_string())?;
    let active = resolve_mode_profile(&policy, active_mode).map_err(|e| e.to_string())?;
    let deltas = json!({
      "max_page_size": {
        "strict": strict.max_page_size,
        "active": active.max_page_size
      },
      "max_region_span": {
        "strict": strict.max_region_span,
        "active": active.max_region_span
      },
      "max_response_bytes": {
        "strict": strict.max_response_bytes,
        "active": active.max_response_bytes
      }
    });
    let payload = json!({
      "command": "atlas policy explain",
      "status": "ok",
      "mode": active_mode.as_str(),
      "strict_mode": "strict",
      "deltas_vs_strict": deltas
    });
    output::emit_ok(output_mode, payload)
}
