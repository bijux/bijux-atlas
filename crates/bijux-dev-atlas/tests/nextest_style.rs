// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::model::engine::{
    CaseReport, CaseStatus, ContractLane, ContractMode, ContractSummary, EffectKind, RunMetadata,
    RunReport, TestKind,
};
use bijux_dev_atlas::ui::terminal::report::{render_status_line, LineStyle};
use bijux_dev_atlas::ui::terminal::nextest_style::{render, PreflightSummary, RenderOptions};
use std::fs;
use std::path::PathBuf;

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("goldens")
        .join(name)
}

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

fn mixed_report() -> RunReport {
    RunReport {
        domain: "ops".to_string(),
        lane: ContractLane::Local,
        mode: bijux_dev_atlas::model::engine::Mode::Static,
        metadata: RunMetadata {
            run_id: "local".to_string(),
            commit_sha: None,
            dirty_tree: false,
            ci: false,
            color_enabled: false,
        },
        contracts: vec![
            ContractSummary {
                id: "OPS-001".to_string(),
                title: "Ops one".to_string(),
                required: false,
                lanes: vec![],
                mode: ContractMode::Static,
                effects: vec![EffectKind::FsWrite],
                status: CaseStatus::Fail,
                duration_ms: 40,
            },
            ContractSummary {
                id: "OPS-002".to_string(),
                title: "Ops two".to_string(),
                required: false,
                lanes: vec![],
                mode: ContractMode::Static,
                effects: vec![EffectKind::FsWrite],
                status: CaseStatus::Skip,
                duration_ms: 20,
            },
        ],
        cases: vec![
            CaseReport {
                contract_id: "OPS-001".to_string(),
                contract_title: "Ops one".to_string(),
                required: false,
                lanes: vec![],
                test_id: "ops.case.pass_a".to_string(),
                test_title: "pass a".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Pass,
                duration_ms: 10,
                violations: vec![],
                note: None,
            },
            CaseReport {
                contract_id: "OPS-001".to_string(),
                contract_title: "Ops one".to_string(),
                required: false,
                lanes: vec![],
                test_id: "ops.case.fail".to_string(),
                test_title: "fail".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Fail,
                duration_ms: 12,
                violations: vec![],
                note: Some("violation".to_string()),
            },
            CaseReport {
                contract_id: "OPS-001".to_string(),
                contract_title: "Ops one".to_string(),
                required: false,
                lanes: vec![],
                test_id: "ops.case.pass_b".to_string(),
                test_title: "pass b".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Pass,
                duration_ms: 8,
                violations: vec![],
                note: None,
            },
            CaseReport {
                contract_id: "OPS-002".to_string(),
                contract_title: "Ops two".to_string(),
                required: false,
                lanes: vec![],
                test_id: "ops.case.skip_a".to_string(),
                test_title: "skip a".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Skip,
                duration_ms: 5,
                violations: vec![],
                note: Some("missing tool: sh".to_string()),
            },
            CaseReport {
                contract_id: "OPS-002".to_string(),
                contract_title: "Ops two".to_string(),
                required: false,
                lanes: vec![],
                test_id: "ops.case.skip_b".to_string(),
                test_title: "skip b".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Skip,
                duration_ms: 5,
                violations: vec![],
                note: Some("disabled effect policy".to_string()),
            },
            CaseReport {
                contract_id: "OPS-002".to_string(),
                contract_title: "Ops two".to_string(),
                required: false,
                lanes: vec![],
                test_id: "ops.case.pass_c".to_string(),
                test_title: "pass c".to_string(),
                kind: TestKind::Pure,
                status: CaseStatus::Pass,
                duration_ms: 7,
                violations: vec![],
                note: None,
            },
        ],
        panics: vec![],
        duration_ms: 60,
    }
}

fn wide_counter_report() -> RunReport {
    let cases = (1..=1002)
        .map(|index| CaseReport {
            contract_id: "OPS-COUNTER-001".to_string(),
            contract_title: "Wide counter".to_string(),
            required: false,
            lanes: vec![],
            test_id: format!("ops.counter.case_{index:04}"),
            test_title: "counter".to_string(),
            kind: TestKind::Pure,
            status: CaseStatus::Pass,
            duration_ms: 1,
            violations: vec![],
            note: None,
        })
        .collect::<Vec<_>>();
    RunReport {
        domain: "ops".to_string(),
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
            id: "OPS-COUNTER-001".to_string(),
            title: "Wide counter".to_string(),
            required: false,
            lanes: vec![],
            mode: ContractMode::Static,
            effects: vec![EffectKind::FsWrite],
            status: CaseStatus::Pass,
            duration_ms: 1002,
        }],
        cases,
        panics: vec![],
        duration_ms: 1002,
    }
}

fn long_name_report() -> RunReport {
    RunReport {
        domain: "governance".to_string(),
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
            id: "GOVERNANCE-CONTRACT-NAME-THAT-STAYS-LONG-AND-EXPLICIT-001".to_string(),
            title: "Long name".to_string(),
            required: false,
            lanes: vec![],
            mode: ContractMode::Static,
            effects: vec![EffectKind::FsWrite],
            status: CaseStatus::Fail,
            duration_ms: 22,
        }],
        cases: vec![CaseReport {
            contract_id: "GOVERNANCE-CONTRACT-NAME-THAT-STAYS-LONG-AND-EXPLICIT-001".to_string(),
            contract_title: "Long name".to_string(),
            required: false,
            lanes: vec![],
            test_id: "governance.case_name_that_stays_long_and_explicit_without_wrapping".to_string(),
            test_title: "long".to_string(),
            kind: TestKind::Pure,
            status: CaseStatus::Fail,
            duration_ms: 22,
            violations: vec![],
            note: Some("long-output".to_string()),
        }],
        panics: vec![],
        duration_ms: 22,
    }
}

#[test]
fn renders_nextest_style_contract_lines() {
    let rendered = render(
        &[sample_report()],
        "static",
        "auto",
        false,
        &PreflightSummary {
            required_tools: vec!["sh".to_string()],
            missing_tools: vec![],
        },
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    assert!(rendered.contains("preflight: required-tools=sh missing-tools=none"));
    assert!(rendered.contains("planning: contracts=1 cases=2"));
    assert!(rendered.contains("FAIL [  0.023s] (1/2) docs::DOC-001 docs.root.links (broken links)"));
    assert!(rendered.contains("PASS [  0.016s] (2/2) docs::DOC-001 docs.root.surface"));
    assert!(rendered.contains("contract-summary: total=2 passed=1 failed=1 skipped=0"));
    assert!(rendered.contains("failed-tests:"));
    assert!(!rendered.contains("\u{1b}["));
}

#[test]
fn shared_line_style_renders_canonical_contract_line() {
    let line = render_status_line(
        LineStyle::Pass,
        false,
        16,
        2,
        12,
        "docs::DOC-001",
        "docs.root.surface",
    );
    assert_eq!(line, "PASS [  0.016s] ( 2/12) docs::DOC-001 docs.root.surface");
}

#[test]
fn matches_no_ansi_golden_output() {
    let rendered = render(
        &[sample_report()],
        "static",
        "auto",
        false,
        &PreflightSummary {
            required_tools: vec!["sh".to_string()],
            missing_tools: vec![],
        },
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    let expected =
        fs::read_to_string(golden_path("contract_runner_no_ansi.txt")).expect("read no-ansi golden");
    assert_eq!(rendered, expected.trim_end());
}

#[test]
fn matches_quiet_golden_output() {
    let rendered = render(
        &[sample_report()],
        "static",
        "auto",
        false,
        &PreflightSummary::default(),
        RenderOptions {
            color: false,
            quiet: true,
            verbose: false,
        },
    );
    let expected =
        fs::read_to_string(golden_path("contract_runner_quiet.txt")).expect("read quiet golden");
    assert_eq!(rendered, expected.trim_end());
}

#[test]
fn matches_verbose_golden_output() {
    let rendered = render(
        &[sample_report()],
        "static",
        "auto",
        false,
        &PreflightSummary {
            required_tools: vec!["sh".to_string()],
            missing_tools: vec![],
        },
        RenderOptions {
            color: false,
            quiet: false,
            verbose: true,
        },
    );
    let expected = fs::read_to_string(golden_path("contract_runner_verbose.txt"))
        .expect("read verbose golden");
    assert_eq!(rendered, expected.trim_end());
}

#[test]
fn matches_mixed_status_golden_output() {
    let rendered = render(
        &[mixed_report()],
        "static",
        "auto",
        false,
        &PreflightSummary {
            required_tools: vec!["sh".to_string()],
            missing_tools: vec!["sh".to_string()],
        },
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    let expected = fs::read_to_string(golden_path("contract_runner_mixed_status.txt"))
        .expect("read mixed-status golden");
    assert_eq!(rendered, expected.trim_end());
}

#[test]
fn keeps_counter_width_stable_above_one_thousand() {
    let rendered = render(
        &[wide_counter_report()],
        "static",
        "auto",
        false,
        &PreflightSummary::default(),
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    let expected = fs::read_to_string(golden_path("contract_runner_counter_width.txt"))
        .expect("read counter-width golden");
    for line in expected.lines() {
        assert!(rendered.contains(line), "missing counter-width line: {line}");
    }
}

#[test]
fn preserves_long_contract_and_case_names() {
    let rendered = render(
        &[long_name_report()],
        "static",
        "auto",
        false,
        &PreflightSummary::default(),
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    let expected = fs::read_to_string(golden_path("contract_runner_long_names.txt"))
        .expect("read long-name golden");
    assert_eq!(rendered, expected.trim_end());
}

#[test]
fn rendering_is_deterministic_for_same_input() {
    let left = render(
        &[mixed_report()],
        "static",
        "auto",
        false,
        &PreflightSummary::default(),
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    let right = render(
        &[mixed_report()],
        "static",
        "auto",
        false,
        &PreflightSummary::default(),
        RenderOptions {
            color: false,
            quiet: false,
            verbose: false,
        },
    );
    assert_eq!(left, right);
}
