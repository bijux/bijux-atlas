// SPDX-License-Identifier: Apache-2.0

use super::super::{canonical_json, OutputMode};
use bijux_atlas_runtime::contracts::errors::ConfigPathScope;
use bijux_atlas_runtime::runtime::config::{resolve_bijux_cache_dir, resolve_bijux_config_path};
use serde_json::json;

pub(crate) fn emit_config_paths(machine_json: bool) -> Result<(), String> {
    let payload = json!({
        "workspace_config": resolve_bijux_config_path(ConfigPathScope::Workspace),
        "user_config": resolve_bijux_config_path(ConfigPathScope::User),
        "cache_dir": resolve_bijux_cache_dir(),
    });
    if machine_json {
        let text = canonical_json::text(&payload)?;
        println!("{text}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

pub(crate) fn print_config(canonical_out: bool, output_mode: OutputMode) -> Result<(), String> {
    let payload = json!({
        "workspace_config": resolve_bijux_config_path(ConfigPathScope::Workspace),
        "user_config": resolve_bijux_config_path(ConfigPathScope::User),
        "cache_dir": resolve_bijux_cache_dir(),
        "env": {
            "BIJUX_LOG_LEVEL": std::env::var("BIJUX_LOG_LEVEL").ok(),
            "BIJUX_CACHE_DIR": std::env::var("BIJUX_CACHE_DIR").ok(),
            "ATLAS_STORE_ROOT": std::env::var("ATLAS_STORE_ROOT").ok(),
        }
    });
    if output_mode.json {
        let text = canonical_json::text(&payload)?;
        println!("{text}");
        return Ok(());
    }
    let text = if canonical_out {
        canonical_json::text(&payload)?
    } else {
        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
    };
    println!("{text}");
    Ok(())
}
