// SPDX-License-Identifier: Apache-2.0

use crate::lifecycle::simulation::context::SimulationCommandRunner;
use crate::lifecycle::simulation::paths::{
    simulation_cluster_config, simulation_cluster_context, simulation_cluster_name,
    write_simulation_report,
};
use serde_json::Value;
use std::path::Path;

fn kind_envelope(
    text: &str,
    status: &str,
    action: &str,
    report_path: &Path,
    details: Value,
) -> Value {
    serde_json::json!({
        "schema_version": 1,
        "text": text,
        "rows": [{
            "schema_version": 1,
            "cluster": "kind",
            "action": action,
            "status": status,
            "report_path": report_path.display().to_string(),
            "details": details
        }],
        "summary": {"total": 1, "errors": if status == "ok" { 0 } else { 1 }, "warnings": 0}
    })
}

fn parse_kind_nodes(stdout: &str) -> Result<Vec<Value>, String> {
    let json: Value = serde_json::from_str(stdout)
        .map_err(|err| format!("failed to parse kubectl nodes json: {err}"))?;
    Ok(json
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| {
            let name = item["metadata"]["name"].as_str().unwrap_or("unknown");
            let ready = item["status"]["conditions"]
                .as_array()
                .is_some_and(|conditions| {
                    conditions.iter().any(|condition| {
                        condition["type"].as_str() == Some("Ready")
                            && condition["status"].as_str() == Some("True")
                    })
                });
            serde_json::json!({"name": name, "ready": ready})
        })
        .collect())
}

pub fn kind_up_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
) -> Result<(Value, i32), String> {
    let config_path = simulation_cluster_config(repo_root);
    let argv = vec![
        "create".to_string(),
        "cluster".to_string(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
        "--config".to_string(),
        config_path.display().to_string(),
    ];
    let result = runner.run("kind", &argv, repo_root);
    let (status, detail) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => {
            if err.contains("already exists") {
                (
                    "ok",
                    serde_json::json!({"detail": "cluster already exists"}),
                )
            } else {
                ("failed", serde_json::json!({"error": err}))
            }
        }
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "up",
        "status": status,
        "details": {
            "cluster_name": simulation_cluster_name(),
            "cluster_config": config_path.display().to_string(),
            "context": simulation_cluster_context(),
            "result": detail
        }
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-kind.json", &payload)?;
    Ok((
        kind_envelope(
            if status == "ok" {
                "kind cluster ready"
            } else {
                "kind cluster failed"
            },
            status,
            "up",
            &report_path,
            payload["details"].clone(),
        ),
        if status == "ok" { 0 } else { 1 },
    ))
}

pub fn kind_down_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
) -> Result<(Value, i32), String> {
    let argv = vec![
        "delete".to_string(),
        "cluster".to_string(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
    ];
    let result = runner.run("kind", &argv, repo_root);
    let (status, detail) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => ("failed", serde_json::json!({"error": err})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "down",
        "status": status,
        "details": {
            "cluster_name": simulation_cluster_name(),
            "result": detail
        }
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-kind.json", &payload)?;
    Ok((
        kind_envelope(
            if status == "ok" {
                "kind cluster deleted"
            } else {
                "kind cluster delete failed"
            },
            status,
            "down",
            &report_path,
            payload["details"].clone(),
        ),
        if status == "ok" { 0 } else { 1 },
    ))
}

pub fn kind_status_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
) -> Result<(Value, i32), String> {
    let argv = vec![
        "--context".to_string(),
        simulation_cluster_context(),
        "get".to_string(),
        "nodes".to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let result = runner.run("kubectl", &argv, repo_root);
    let (status, details) = match result {
        Ok((stdout, event)) => {
            let nodes = parse_kind_nodes(&stdout)?;
            ("ok", serde_json::json!({"event": event, "nodes": nodes}))
        }
        Err(err) => ("failed", serde_json::json!({"error": err})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "status",
        "status": status,
        "details": details
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-kind.json", &payload)?;
    Ok((
        kind_envelope(
            if status == "ok" {
                "kind cluster status collected"
            } else {
                "kind cluster status failed"
            },
            status,
            "status",
            &report_path,
            payload["details"].clone(),
        ),
        if status == "ok" { 0 } else { 1 },
    ))
}

pub fn kind_preload_payload(
    runner: &impl SimulationCommandRunner,
    repo_root: &Path,
    run_id: &str,
    image: &str,
) -> Result<(Value, i32), String> {
    let argv = vec![
        "load".to_string(),
        "docker-image".to_string(),
        image.to_string(),
        "--name".to_string(),
        simulation_cluster_name().to_string(),
    ];
    let result = runner.run("kind", &argv, repo_root);
    let (status, details) = match result {
        Ok((stdout, event)) => ("ok", serde_json::json!({"stdout": stdout, "event": event})),
        Err(err) => ("failed", serde_json::json!({"error": err})),
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "cluster": "kind",
        "action": "preload-image",
        "status": status,
        "details": {
            "image": image,
            "result": details
        }
    });
    let report_path = write_simulation_report(repo_root, run_id, "ops-kind.json", &payload)?;
    Ok((
        kind_envelope(
            if status == "ok" {
                "kind image preload complete"
            } else {
                "kind image preload failed"
            },
            status,
            "preload-image",
            &report_path,
            payload["details"].clone(),
        ),
        if status == "ok" { 0 } else { 1 },
    ))
}

#[cfg(test)]
mod tests {
    use super::{kind_status_payload, kind_up_payload};
    use crate::lifecycle::simulation::context::SimulationCommandRunner;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::Path;

    struct MockRunner {
        results: RefCell<VecDeque<Result<(String, Value), String>>>,
    }

    impl SimulationCommandRunner for MockRunner {
        fn run(
            &self,
            _binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<(String, Value), String> {
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    #[test]
    fn kind_up_treats_existing_cluster_as_owned_success() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Err(
                "kind create cluster failed: node(s) already exists for a cluster with the name \"bijux-atlas-kind\"".to_string(),
            )])),
        };

        let (payload, exit_code) =
            kind_up_payload(&runner, root.path(), "atlas-run").expect("kind up payload");

        assert_eq!(exit_code, 0);
        assert_eq!(payload["rows"][0]["status"], "ok");
    }

    #[test]
    fn kind_status_reports_node_readiness_rows() {
        let root = tempfile::tempdir().expect("tempdir");
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok((
                serde_json::json!({
                    "items": [{
                        "metadata": {"name": "atlas-worker"},
                        "status": {"conditions": [{"type": "Ready", "status": "True"}]}
                    }]
                })
                .to_string(),
                serde_json::json!({"binary":"kubectl"}),
            ))])),
        };

        let (payload, exit_code) =
            kind_status_payload(&runner, root.path(), "atlas-run").expect("kind status payload");

        assert_eq!(exit_code, 0);
        assert_eq!(
            payload["rows"][0]["details"]["nodes"][0]["name"],
            "atlas-worker"
        );
        assert_eq!(payload["rows"][0]["details"]["nodes"][0]["ready"], true);
    }
}
