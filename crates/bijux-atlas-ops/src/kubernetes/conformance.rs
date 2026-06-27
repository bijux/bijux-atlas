// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;

pub fn conformance_summary(deployments: &Value, pods: &Value) -> (Vec<String>, Vec<Value>) {
    let mut errors = Vec::new();
    let mut rows = Vec::new();
    if let Some(items) = deployments.get("items").and_then(Value::as_array) {
        for item in items {
            let name = item
                .get("metadata")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let desired = item
                .get("status")
                .and_then(|value| value.get("replicas"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let ready = item
                .get("status")
                .and_then(|value| value.get("readyReplicas"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            if ready < desired {
                errors.push(format!("deployment `{name}` ready {ready}/{desired}"));
            }
            rows.push(serde_json::json!({
                "kind":"deployment",
                "name":name,
                "desired":desired,
                "ready":ready
            }));
        }
    }
    if let Some(items) = pods.get("items").and_then(Value::as_array) {
        for item in items {
            let name = item
                .get("metadata")
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let phase = item
                .get("status")
                .and_then(|value| value.get("phase"))
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            if phase != "Running" && phase != "Succeeded" {
                errors.push(format!("pod `{name}` phase={phase}"));
            }
            rows.push(serde_json::json!({
                "kind":"pod",
                "name":name,
                "phase":phase
            }));
        }
    }
    (errors, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformance_summary_flags_unready_workloads() {
        let deployments = serde_json::json!({
            "items":[{"metadata":{"name":"atlas"},"status":{"replicas":2,"readyReplicas":1}}]
        });
        let pods = serde_json::json!({
            "items":[{"metadata":{"name":"atlas-1"},"status":{"phase":"Pending"}}]
        });

        let (errors, rows) = conformance_summary(&deployments, &pods);

        assert_eq!(rows.len(), 2);
        assert!(errors.iter().any(|entry| entry.contains("deployment")));
        assert!(errors.iter().any(|entry| entry.contains("pod")));
    }
}
