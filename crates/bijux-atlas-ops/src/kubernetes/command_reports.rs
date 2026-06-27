// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn k8s_plan_payload(
    profile: &str,
    run_id: &str,
    render_path: &str,
    render_index_path: &str,
    index: Value,
) -> Value {
    json!({
        "schema_version": 1,
        "text": format!("k8s plan profile={profile} run_id={run_id}"),
        "rows": [{
            "profile": profile,
            "run_id": run_id,
            "render_path": render_path,
            "render_index_path": render_index_path,
            "index": index
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    })
}

pub fn k8s_apply_payload(
    profile: &str,
    run_id: &str,
    dry_run: bool,
    render_path: &str,
    stdout: &str,
    subprocess_event: Value,
) -> Value {
    json!({
        "schema_version": 1,
        "text": if dry_run { "k8s dry-run completed" } else { "k8s apply completed" },
        "rows": [{
            "profile": profile,
            "run_id": run_id,
            "dry_run": dry_run,
            "render_path": render_path,
            "stdout": stdout,
            "subprocess_event": subprocess_event
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    })
}

pub fn k8s_logs_payload(stdout: &str, subprocess_event: Value) -> Value {
    json!({
        "schema_version": 1,
        "text": "k8s logs collected",
        "rows": [{
            "stdout": stdout,
            "event": subprocess_event
        }],
        "summary": {"total":1,"errors":0,"warnings":0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_payload_preserves_render_pointers() {
        let payload = k8s_plan_payload(
            "kind",
            "atlas-run",
            "artifacts/ops/atlas-run/render/kind/helm/render.yaml",
            "artifacts/ops/atlas-run/render/kind/helm/render.index.json",
            json!({"files":[]}),
        );

        assert_eq!(payload["rows"][0]["profile"], "kind");
        assert_eq!(payload["rows"][0]["run_id"], "atlas-run");
        assert!(payload["text"]
            .as_str()
            .is_some_and(|text| text.contains("k8s plan")));
    }

    #[test]
    fn apply_payload_tracks_dry_run_status() {
        let payload = k8s_apply_payload(
            "kind",
            "atlas-run",
            true,
            "render.yaml",
            "applied",
            json!({"binary":"kubectl"}),
        );

        assert_eq!(payload["text"], "k8s dry-run completed");
        assert_eq!(payload["rows"][0]["dry_run"], true);
    }

    #[test]
    fn logs_payload_wraps_stdout_and_event() {
        let payload = k8s_logs_payload("hello", json!({"binary":"kubectl"}));
        assert_eq!(payload["text"], "k8s logs collected");
        assert_eq!(payload["rows"][0]["stdout"], "hello");
        assert_eq!(payload["rows"][0]["event"]["binary"], "kubectl");
    }
}
