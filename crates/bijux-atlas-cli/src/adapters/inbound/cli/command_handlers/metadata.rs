// SPDX-License-Identifier: Apache-2.0

use super::super::{canonical_json, CliError, ExitCode, MachineError, OutputMode};
use super::super::{UMBRELLA_MAX_EXCLUSIVE_VERSION, UMBRELLA_MIN_VERSION};
use crate::adapters::inbound::cli::output;
use bijux_atlas_runtime::runtime::config::runtime_build_hash;
use bijux_atlas_runtime::version::{runtime_semver, runtime_version, runtime_version_source};
use serde_json::{json, Value};

pub(crate) fn emit_plugin_metadata(machine_json: bool) -> Result<(), String> {
    let payload = plugin_metadata_payload();

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

pub(crate) fn print_version(verbose: bool, output_mode: OutputMode) -> Result<(), String> {
    let payload = if verbose {
        json!({
                "plugin": {
                    "name": "bijux-atlas",
                    "version": runtime_version(),
                    "semver": runtime_semver(),
                    "source": runtime_version_source(),
                    "build_hash": runtime_build_hash(),
                    "rustc": option_env!("RUSTC_VERSION").unwrap_or("unknown")
                },
            "schemas": {
                "plugin_metadata_schema_version": "v1",
                "openapi_version": "v1"
            }
        })
    } else {
        json!({"name":"bijux-atlas","version": runtime_version()})
    };
    output::emit_ok(output_mode, payload)?;
    Ok(())
}

pub(crate) fn enforce_umbrella_compatibility(version: &str) -> Result<(), CliError> {
    if !version_in_supported_range(version) {
        return Err(CliError {
            exit_code: ExitCode::Usage,
            machine: MachineError::new(
                "umbrella_incompatible",
                "umbrella version is outside plugin compatibility range",
            )
            .with_detail("version", version)
            .with_detail("min", UMBRELLA_MIN_VERSION)
            .with_detail("max_exclusive", UMBRELLA_MAX_EXCLUSIVE_VERSION),
        });
    }
    Ok(())
}

fn plugin_metadata_payload() -> Value {
    json!({
        "schema_version": "v1",
        "name": "bijux-atlas",
        "version": runtime_semver(),
        "version_display": runtime_version(),
        "compatible_umbrella_min": UMBRELLA_MIN_VERSION,
        "compatible_umbrella_max_exclusive": UMBRELLA_MAX_EXCLUSIVE_VERSION,
        "compatible_umbrella": ">=0.3.0,<0.4.0",
        "build_hash": runtime_build_hash(),
    })
}

fn version_in_supported_range(version: &str) -> bool {
    let parts: Vec<_> = version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    matches!((parts[0], parts[1]), ("0", "3"))
}
