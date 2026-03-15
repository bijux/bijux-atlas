// SPDX-License-Identifier: Apache-2.0

use crate::*;
use bijux_dev_atlas::policies::DevAtlasPolicySet;

fn policies_inventory_rows(
    doc: &bijux_dev_atlas::policies::DevAtlasPolicySetDocument,
) -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "repo",
            "title": "repository structure and budget policy",
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "ops",
            "title": "ops registry policy",
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "compatibility",
            "title": "policy mode compatibility matrix",
            "count": doc.compatibility.len(),
            "schema_version": doc.schema_version.as_str()
        }),
        serde_json::json!({
            "id": "documented_defaults",
            "title": "documented default exceptions",
            "count": doc.documented_defaults.len(),
            "schema_version": doc.schema_version.as_str()
        }),
    ]
}

pub(crate) fn run_policies_list(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let rows = policies_inventory_rows(&doc);
    let payload = serde_json::json!({
        "schema_version": 1,
        "repo_root": root.display().to_string(),
        "rows": rows,
        "text": "control-plane policies listed"
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_explain(
    policy_id: String,
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let payload = match policy_id.as_str() {
        "repo" => serde_json::json!({
            "schema_version": 1,
            "id": "repo",
            "repo_root": root.display().to_string(),
            "title": "repository structure and budget policy",
            "fields": doc.repo,
        }),
        "ops" => serde_json::json!({
            "schema_version": 1,
            "id": "ops",
            "repo_root": root.display().to_string(),
            "title": "ops registry policy",
            "fields": doc.ops,
        }),
        "compatibility" => serde_json::json!({
            "schema_version": 1,
            "id": "compatibility",
            "repo_root": root.display().to_string(),
            "title": "policy mode compatibility matrix",
            "rows": doc.compatibility,
        }),
        "documented_defaults" => serde_json::json!({
            "schema_version": 1,
            "id": "documented_defaults",
            "repo_root": root.display().to_string(),
            "title": "documented default exceptions",
            "rows": doc.documented_defaults,
        }),
        _ => {
            return Err(format!(
                "unknown policy id `{}` (expected repo|ops|compatibility|documented_defaults)",
                policy_id
            ))
        }
    };
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_policies_report(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let policies = DevAtlasPolicySet::load(&root).map_err(|err| err.to_string())?;
    let doc = policies.to_document();
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "repo_root": root.display().to_string(),
        "policy_schema_version": doc.schema_version.as_str(),
        "mode": format!("{:?}", doc.mode).to_ascii_lowercase(),
        "policy_count": policies_inventory_rows(&doc).len(),
        "capabilities": {"fs_write": false, "subprocess": false, "network": false, "git": false},
        "report_kind": "control_plane_policies"
    });
    let rendered = emit_payload(format, out, &payload)?;
    Ok((rendered, 0))
}
