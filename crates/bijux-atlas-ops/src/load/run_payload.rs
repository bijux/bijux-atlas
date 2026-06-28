// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn load_run_payload(
    suite: &str,
    run_id: &str,
    stdout: &str,
    subprocess_event: Value,
    summary_path: &str,
    report: Value,
    report_code: i32,
) -> Value {
    json!({
        "schema_version": 1,
        "text": format!("ops load run suite={suite}"),
        "rows": [{
            "suite": suite,
            "run_id": run_id,
            "k6_stdout": stdout,
            "subprocess_event": subprocess_event,
            "summary_path": summary_path,
            "report": report
        }],
        "summary": {"total": 1, "errors": if report_code == 0 { 0 } else { 1 }, "warnings": 0}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_run_payload_reflects_report_failure() {
        let payload = load_run_payload(
            "mixed",
            "atlas-run",
            "k6 output",
            json!({"binary":"k6"}),
            "artifacts/ops/atlas-run/load/mixed/k6-summary.json",
            json!({"violations":["threshold breach"]}),
            1,
        );

        assert_eq!(payload["text"], "ops load run suite=mixed");
        assert_eq!(payload["summary"]["errors"], 1);
        assert_eq!(payload["rows"][0]["subprocess_event"]["binary"], "k6");
    }
}
