// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use bijux_dev_atlas::core::ops_registry::builtin_ops_registry;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn generated_ops_surface_snapshot_matches_registry_entries() {
    let root = workspace_root();
    let snapshot_path = root.join("ops/_generated.example/control-plane-surface-list.json");
    let text = fs::read_to_string(&snapshot_path).expect("read control-plane surface snapshot");
    let json: serde_json::Value = serde_json::from_str(&text).expect("parse surface snapshot");

    let snapshot_entries: BTreeSet<(String, String, Option<String>)> = json
        .get("ops_taxonomy")
        .and_then(|v| v.get("entries"))
        .and_then(|v| v.as_array())
        .expect("ops_taxonomy.entries array")
        .iter()
        .map(|entry| {
            let domain = entry
                .get("domain")
                .and_then(|v| v.as_str())
                .expect("entry domain")
                .to_string();
            let verb = entry
                .get("verb")
                .and_then(|v| v.as_str())
                .expect("entry verb")
                .to_string();
            let subverb = entry
                .get("subverb")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            (domain, verb, subverb)
        })
        .collect();

    let registry_entries: BTreeSet<(String, String, Option<String>)> = builtin_ops_registry()
        .into_iter()
        .map(|entry| {
            (
                entry.domain.to_string(),
                entry.verb.to_string(),
                entry.subverb.map(ToString::to_string),
            )
        })
        .collect();

    assert_eq!(
        registry_entries, snapshot_entries,
        "control-plane-surface-list snapshot must match builtin ops registry exactly"
    );
}

#[test]
fn file_usage_report_has_no_orphan_ops_files() {
    let root = workspace_root();
    let report_path = root.join("ops/_generated.example/file-usage-report.json");
    let text = fs::read_to_string(&report_path).expect("read file usage report");
    let json: serde_json::Value = serde_json::from_str(&text).expect("parse file usage report");
    let orphans = json
        .get("orphans")
        .and_then(|v| v.as_array())
        .expect("orphans array");
    assert!(
        orphans.is_empty(),
        "file usage report contains orphan ops files: {orphans:?}"
    );
}
