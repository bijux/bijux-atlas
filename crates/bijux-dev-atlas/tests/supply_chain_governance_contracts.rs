// SPDX-License-Identifier: Apache-2.0

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
fn supply_chain_docs_reports_and_workflows_are_present() {
    let root = workspace_root();

    let required_docs = [
        "docs/operations/security/supply-chain-security-policy.md",
        "docs/operations/security/dependency-trust-model.md",
        "docs/operations/security/supply-chain-documentation.md",
        "docs/operations/security/supply-chain-architecture-diagram.md",
        "docs/operations/security/supply-chain-ci-validation.md",
        "docs/operations/security/supply-chain-delivery-report.md",
    ];
    for path in required_docs {
        assert!(root.join(path).exists(), "missing required doc: {path}");
    }

    let required_reports = [
        "ops/security/reports/dependency-metrics.example.json",
        "ops/security/reports/vulnerability-severity-dashboard.example.json",
        "ops/security/reports/supply-chain-compliance-report.example.json",
        "ops/security/reports/dependency-risk-scoring-report.example.json",
        "ops/security/reports/supply-chain-risk-dashboard.example.json",
    ];
    for path in required_reports {
        assert!(root.join(path).exists(), "missing required report: {path}");
    }

    assert!(
        root.join(".github/workflows/security-supply-chain-validation.yml")
            .exists(),
        "missing supply chain validation workflow"
    );
}
