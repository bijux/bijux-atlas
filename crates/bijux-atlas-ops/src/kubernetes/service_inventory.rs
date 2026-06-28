// SPDX-License-Identifier: Apache-2.0

use crate::kubernetes::execution::KubernetesCommandRunner;
use serde_json::{json, Value};
use std::path::Path;

pub fn service_port_rows(services: &Value) -> Vec<Value> {
    services
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|service| {
            let name = service
                .get("metadata")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let cluster_ip = service
                .get("spec")
                .and_then(|value| value.get("clusterIP"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let ports = service
                .get("spec")
                .and_then(|value| value.get("ports"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "kind":"service_port_discovery",
                "service": name,
                "cluster_ip": cluster_ip,
                "ports": ports
            })
        })
        .collect()
}

pub fn read_service_port_rows(
    runner: &impl KubernetesCommandRunner,
    repo_root: &Path,
    namespace: &str,
) -> Result<(Vec<Value>, Value), String> {
    let args = vec![
        "get".to_string(),
        "service".to_string(),
        "-n".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "json".to_string(),
    ];
    let capture = runner.run("kubectl", &args, repo_root)?;
    let services: Value = serde_json::from_str(&capture.stdout)
        .map_err(|err| format!("failed parsing service json: {err}"))?;
    Ok((service_port_rows(&services), capture.event))
}

pub fn service_port_payload(rows: Vec<Value>, event: Value) -> Value {
    json!({
        "schema_version":1,
        "text":"k8s ports discovery complete",
        "rows": rows,
        "subprocess_events":[event],
        "summary":{"total":1,"errors":0,"warnings":0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::execution::SubprocessCapture;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::path::Path;

    struct MockRunner {
        results: RefCell<VecDeque<Result<SubprocessCapture, String>>>,
    }

    impl KubernetesCommandRunner for MockRunner {
        fn run(
            &self,
            binary: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<SubprocessCapture, String> {
            assert_eq!(binary, "kubectl");
            self.results
                .borrow_mut()
                .pop_front()
                .expect("mock result should exist")
        }
    }

    #[test]
    fn service_inventory_reports_cluster_ip_and_ports() {
        let services = serde_json::json!({
            "items": [{
                "metadata": { "name": "atlas-api" },
                "spec": {
                    "clusterIP": "10.0.0.15",
                    "ports": [{ "name": "http", "port": 8080 }]
                }
            }]
        });

        let rows = service_port_rows(&services);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["kind"], "service_port_discovery");
        assert_eq!(rows[0]["service"], "atlas-api");
        assert_eq!(rows[0]["cluster_ip"], "10.0.0.15");
        assert_eq!(rows[0]["ports"][0]["port"], 8080);
    }

    #[test]
    fn read_service_port_rows_collects_owner_rows_and_event() {
        let runner = MockRunner {
            results: RefCell::new(VecDeque::from([Ok(SubprocessCapture {
                stdout: serde_json::json!({
                    "items": [{
                        "metadata": { "name": "atlas-api" },
                        "spec": {
                            "clusterIP": "10.0.0.15",
                            "ports": [{ "name": "http", "port": 8080 }]
                        }
                    }]
                })
                .to_string(),
                event: json!({"binary":"kubectl"}),
            })])),
        };

        let (rows, event) = read_service_port_rows(&runner, Path::new("/repo"), "bijux-atlas")
            .expect("service rows should load");

        assert_eq!(rows.len(), 1);
        assert_eq!(event["binary"], "kubectl");
        assert_eq!(rows[0]["service"], "atlas-api");
    }

    #[test]
    fn service_port_payload_wraps_rows_and_events() {
        let payload = service_port_payload(
            vec![json!({"service":"atlas-api"})],
            json!({"binary":"kubectl"}),
        );

        assert_eq!(payload["text"], "k8s ports discovery complete");
        assert_eq!(payload["rows"][0]["service"], "atlas-api");
        assert_eq!(payload["subprocess_events"][0]["binary"], "kubectl");
    }
}
