// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn data_protection_docs_and_reports_are_governed() {
    let root = workspace_root();

    let required_docs = [
        "docs/operations/security/data-protection-documentation.md",
        "docs/operations/security/encryption-configuration-guide.md",
        "docs/operations/security/artifact-integrity-troubleshooting-guide.md",
        "docs/operations/security/secure-deployment-guide.md",
        "docs/operations/security/encryption-compliance-checklist.md",
        "docs/operations/security/data-protection-policy-reference.md",
    ];
    for doc in required_docs {
        assert!(root.join(doc).exists(), "missing required doc: {doc}");
    }

    let required_reports = [
        "ops/security/reports/data-protection-evidence.example.json",
        "ops/security/reports/encryption-configuration-audit-report.example.json",
        "ops/security/reports/artifact-integrity-audit-report.example.json",
        "ops/security/reports/tamper-detection-audit-report.example.json",
    ];
    for report in required_reports {
        let path = root.join(report);
        assert!(path.exists(), "missing required report: {report}");
        let content = fs::read_to_string(&path).expect("read report");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("parse report json");
        assert_eq!(parsed["schema_version"], serde_json::json!(1));
    }
}
