// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn ops_list_payload(profile_count: usize, mut action_ids: Vec<String>) -> Value {
    action_ids.sort();
    let rows = vec![
        json!({"kind":"capability","name":"inventory","subprocess":false,"write":false}),
        json!({"kind":"capability","name":"validate","subprocess":false,"write":false}),
        json!({"kind":"capability","name":"render","subprocess":true,"write":"flag_gated"}),
        json!({"kind":"capability","name":"install","subprocess":true,"write":"flag_gated"}),
        json!({"kind":"capability","name":"status","subprocess":"target_gated","write":false}),
        json!({"kind":"capability","name":"cleanup","subprocess":"profile_dependent","write":false}),
        json!({"kind":"profiles","count": profile_count}),
        json!({"kind":"actions","count": action_ids.len(), "action_ids": action_ids}),
    ];

    json!({
        "schema_version": 1,
        "text": "ops list capabilities and actions",
        "rows": rows,
        "summary": {"total": 8, "errors": 0, "warnings": 0}
    })
}

pub fn ops_explain_payload(action: &str) -> Result<Value, String> {
    let action_lc = action.trim().to_ascii_lowercase();
    let row = match action_lc.as_str() {
        "kind-up" => json!({"action":"kind-up","purpose":"create the deterministic kind simulation cluster","effects_required":["subprocess","fs_write"]}),
        "kind-down" => json!({"action":"kind-down","purpose":"delete the deterministic kind simulation cluster","effects_required":["subprocess"]}),
        "kind-status" => json!({"action":"kind-status","purpose":"report whether the deterministic kind simulation cluster is reachable and ready","effects_required":["subprocess"]}),
        "inventory" => json!({"action":"inventory","purpose":"list ops manifests and inventory validity","effects_required":[]}),
        "validate" => json!({"action":"validate","purpose":"validate ops SSOT inputs and checks","effects_required":[]}),
        "render" | "k8s-render" => json!({"action":"render","purpose":"render deterministic ops manifests","effects_required":["subprocess"],"flags":["--allow-subprocess","--allow-write"]}),
        "k8s-plan" => json!({"action":"k8s-plan","purpose":"show what rendered resources would be applied","effects_required":[]}),
        "stack-plan" => json!({"action":"stack-plan","purpose":"resolve stack resources for a profile without executing subprocesses","effects_required":[]}),
        "install" | "stack-up" => json!({"action":"install","purpose":"plan/apply ops stack to local cluster","effects_required":["subprocess","fs_write","network"],"flags":["--allow-subprocess","--allow-write","--allow-network"]}),
        "down" | "stack-down" => json!({"action":"down","purpose":"teardown local ops stack resources","effects_required":["subprocess"],"flags":["--allow-subprocess"]}),
        "status" | "stack-status" => json!({"action":"status","purpose":"collect local/k8s status rows","effects_required":["subprocess (for k8s/pods/endpoints)"]}),
        "stack-versions" => json!({"action":"stack-versions","purpose":"emit deterministic stack component version inventory","effects_required":["fs_read"],"output":"versions.json"}),
        "conformance" | "k8s-test" => json!({"action":"conformance","purpose":"run ops conformance status checks","effects_required":["subprocess"],"flags":["--allow-subprocess"]}),
        "k8s-smoke" => json!({"action":"k8s-smoke","purpose":"run cluster smoke checks against health/query paths","effects_required":["subprocess","network"],"flags":["--allow-subprocess","--allow-network"]}),
        "smoke" => json!({"action":"smoke","purpose":"run simulation smoke checks against /healthz, /readyz, and /v1/version","effects_required":["subprocess","network"],"flags":["--allow-subprocess","--allow-network"]}),
        "k8s-ports" => json!({"action":"k8s-ports","purpose":"discover service and endpoint ports for evidence collection","effects_required":["subprocess"],"flags":["--allow-subprocess"]}),
        "load-plan" => json!({"action":"load-plan","purpose":"resolve load suite to script env and thresholds","effects_required":[]}),
        "load-run" => json!({"action":"load-run","purpose":"run k6 load suite and collect summary","effects_required":["subprocess","network","fs_write"]}),
        "load-report" => json!({"action":"load-report","purpose":"parse k6 summary into structured report","effects_required":[]}),
        "e2e-run" => json!({"action":"e2e-run","purpose":"reserved for scenario orchestration","status":"not_implemented"}),
        "obs-drill-run" | "drills-run" => json!({"action":"drills-run","purpose":"run a governed institutional drill and emit a drill report","effects_required":["fs_write"],"flags":["--allow-write","--name <drill>"]}),
        "obs-verify" => json!({"action":"obs-verify","purpose":"verify metrics endpoint reachability and required observability contracts","effects_required":["subprocess","network","fs_write"],"flags":["--allow-subprocess","--allow-network","--allow-write"]}),
        "tools-doctor" => json!({"action":"tools-doctor","purpose":"show required tools and missing requirements without subprocess by default","effects_required":[]}),
        "suite-list" => json!({"kind":"suite","action":"list","suites":["e2e","k8s","load","obs"]}),
        value if value.starts_with("suite-run:") => json!({"kind":"suite","action":"run","suite":value.trim_start_matches("suite-run:")}),
        "cleanup" => json!({"action":"cleanup","purpose":"remove scoped artifacts and local ops resources","effects_required":["subprocess (optional)"]}),
        _ => {
            return Err(format!(
                "unknown ops action `{}` (try inventory|validate|render|install|down|status|conformance|cleanup|load-plan|load-run|load-report|e2e-run|obs-drill-run)",
                action
            ))
        }
    };

    Ok(json!({
        "schema_version": 1,
        "text": format!("ops explain {}", action_lc),
        "rows": [row],
        "summary": {"total": 1, "errors": 0, "warnings": 0}
    }))
}

#[cfg(test)]
mod tests {
    use super::{ops_explain_payload, ops_list_payload};

    #[test]
    fn ops_list_payload_sorts_actions_and_reports_counts() {
        let payload = ops_list_payload(
            3,
            vec![
                "stack-down".to_string(),
                "inventory".to_string(),
                "stack-up".to_string(),
            ],
        );

        assert_eq!(payload["summary"]["total"], 8);
        assert_eq!(payload["rows"][6]["count"], 3);
        assert_eq!(
            payload["rows"][7]["action_ids"],
            serde_json::json!(["inventory", "stack-down", "stack-up"])
        );
    }

    #[test]
    fn ops_explain_payload_preserves_suite_run_subject() {
        let payload = ops_explain_payload("suite-run:load").expect("payload");

        assert_eq!(payload["text"], "ops explain suite-run:load");
        assert_eq!(payload["rows"][0]["kind"], "suite");
        assert_eq!(payload["rows"][0]["suite"], "load");
    }

    #[test]
    fn ops_explain_payload_rejects_unknown_actions() {
        let error = ops_explain_payload("unknown-action").expect_err("unknown action");

        assert!(error.contains("unknown ops action `unknown-action`"));
    }
}
