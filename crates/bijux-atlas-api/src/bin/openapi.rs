// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let mut out: Option<PathBuf> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--out" {
            out = args.next().map(PathBuf::from);
        }
    }
    let out = out.ok_or_else(|| "missing --out <path>".to_string())?;

    let spec = bijux_atlas_api::openapi_v1_spec();
    let bytes = bijux_atlas_core::canonical::stable_json_bytes(&spec).map_err(|e| e.to_string())?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&out, bytes).map_err(|e| e.to_string())?;
    println!("wrote OpenAPI spec: {}", out.display());
    Ok(())
}
