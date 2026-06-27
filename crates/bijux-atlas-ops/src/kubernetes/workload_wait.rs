// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn readiness_wait_commands(namespace: &str, timeout_seconds: u64) -> Vec<Vec<String>> {
    let timeout = format!("{timeout_seconds}s");
    vec![
        vec![
            "wait".to_string(),
            "deployment".to_string(),
            "--all".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--for=condition=Available".to_string(),
            format!("--timeout={timeout}"),
        ],
        vec![
            "wait".to_string(),
            "pod".to_string(),
            "--all".to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--for=condition=Ready".to_string(),
            format!("--timeout={timeout}"),
        ],
    ]
}

pub fn readiness_wait_success_row(argv: &[String], stdout: &str, event: Value) -> Value {
    json!({
        "argv": argv,
        "stdout": stdout,
        "event": event,
        "status": "ok"
    })
}

pub fn readiness_wait_failure_row(argv: &[String]) -> Value {
    json!({
        "argv": argv,
        "status": "failed"
    })
}

pub fn readiness_wait_payload(rows: Vec<Value>, errors: &[String], elapsed_ms: u128) -> Value {
    json!({
        "schema_version": 1,
        "text": if errors.is_empty() { "k8s wait passed" } else { "k8s wait failed" },
        "rows": rows,
        "errors": errors,
        "summary": { "total": 1, "errors": errors.len(), "warnings": 0 },
        "elapsed_ms": elapsed_ms
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_wait_commands_cover_deployments_and_pods() {
        let commands = readiness_wait_commands("bijux-atlas", 90);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0][0], "wait");
        assert!(commands[0].contains(&"deployment".to_string()));
        assert!(commands[1].contains(&"pod".to_string()));
        assert!(commands[0].contains(&"--timeout=90s".to_string()));
    }

    #[test]
    fn readiness_wait_payload_tracks_failures() {
        let payload = readiness_wait_payload(
            vec![readiness_wait_failure_row(&[
                "wait".to_string(),
                "pod".to_string(),
            ])],
            &["pod wait failed".to_string()],
            1200,
        );
        assert_eq!(payload["text"], "k8s wait failed");
        assert_eq!(payload["summary"]["errors"], 1);
        assert_eq!(payload["elapsed_ms"], 1200);
    }
}
