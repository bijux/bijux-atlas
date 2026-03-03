// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

use bijux_dev_atlas::engine::{encode_pretty, read_json, validate_run_result_report, write_json};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bijux-dev-atlas-{}-{}-{}.json",
        name,
        std::process::id(),
        std::thread::current().name().unwrap_or("main")
    ))
}

#[test]
fn report_codec_writes_and_reads_canonical_json() {
    let path = temp_path("report-codec");
    let payload = serde_json::json!({
      "report_id": "run-result",
      "version": 1,
      "inputs": {"suite": "checks"},
      "artifacts": [],
      "run_id": "run-1",
      "runnable_id": "CHECK-001",
      "status": "pass"
    });
    write_json(&path, &payload).expect("write report");
    let text = fs::read_to_string(&path).expect("read report text");
    assert!(text.ends_with('\n'));
    let encoded = encode_pretty(&payload).expect("encode report");
    assert_eq!(text, format!("{encoded}\n"));
    let roundtrip = read_json(&path).expect("roundtrip report");
    assert_eq!(roundtrip, payload);
    let _ = fs::remove_file(&path);
}

#[test]
fn run_result_report_requires_shared_header_fields() {
    let payload = serde_json::json!({
      "report_id": "run-result",
      "version": 1,
      "inputs": {"suite": "checks"},
      "artifacts": [],
      "run_id": "run-1",
      "runnable_id": "CHECK-001",
      "status": "pass"
    });
    validate_run_result_report("run-result", &payload).expect("valid report");
}
