// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::model::engine::{
    CaseReport, CaseStatus, ContractLane, ContractMode, ContractSummary, EffectKind, RunMetadata,
    RunReport, TestKind,
};
use bijux_dev_atlas::ui::terminal::nextest_style::{render, RenderOptions};

fn sample_report() -> RunReport {
    RunReport {
        domain: "docs".to_string(),
        lane: ContractLane::Local,
        mode: bijux_dev_atlas::model::engine::Mode::Static,
        metadata: RunMetadata {
            run_id: "local".to_string(),
            commit_sha: None,
            dirty_tree: false,
            ci: false,
            color_enabled: false,
        },
        contracts: vec![ContractSummary {
            id: "DOC-001".to_string(),
            title: "Docs root".to_string(),
            required: false,
            lanes: vec![],
            mode: ContractMode::Static,
            effects: vec![EffectKind::FsWrite],
            status: CaseStatus::Fail,
            duration_ms: 16,
        }],
        cases: vec![
            CaseReport {
                contract_id: "DOC-001".to_string(),
                contract_title: "Docs root".to_string(),
                required: false,
                lanes: vec![],
                test_id: "docs.root.surface".to_string(),
                test_title: "docs root".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Pass,
                duration_ms: 16,
                violations: vec![],
                note: None,
            },
            CaseReport {
                contract_id: "DOC-001".to_string(),
                contract_title: "Docs root".to_string(),
                required: false,
                lanes: vec![],
                test_id: "docs.root.links".to_string(),
                test_title: "docs links".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Fail,
                duration_ms: 23,
                violations: vec![],
                note: Some("broken links".to_string()),
            },
        ],
        panics: vec![],
        duration_ms: 39,
    }
}

#[test]
fn renders_nextest_style_contract_lines() {
    let rendered = render(
        &[sample_report()],
        "static",
        "auto",
        false,
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    assert!(rendered.contains("FAIL [  0.023s] (1/2) docs::DOC-001 docs.root.links (broken links)"));
    assert!(rendered.contains("PASS [  0.016s] (2/2) docs::DOC-001 docs.root.surface"));
    assert!(rendered.contains("contract-summary: total=2 passed=1 failed=1 skipped=0"));
    assert!(rendered.contains("failed-tests:"));
}
