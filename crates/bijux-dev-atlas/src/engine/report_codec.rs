// SPDX-License-Identifier: Apache-2.0
//! Canonical JSON report encoding for engine-managed artifacts.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

pub fn encode_pretty(value: &serde_json::Value) -> Result<String, String> {
    let mut out = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut serializer = serde_json::Serializer::with_formatter(&mut out, formatter);
    value
        .serialize(&mut serializer)
        .map_err(|err| format!("encode report failed: {err}"))?;
    String::from_utf8(out).map_err(|err| format!("encode report failed: {err}"))
}

pub fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let rendered = encode_pretty(value)?;
    let mut file =
        fs::File::create(path).map_err(|err| format!("write {} failed: {err}", path.display()))?;
    file.write_all(rendered.as_bytes())
        .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    file.write_all(b"\n")
        .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    Ok(())
}

pub fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))
}
