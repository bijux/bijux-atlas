// SPDX-License-Identifier: Apache-2.0
//! Nextest-style contract runner formatting.

use crate::model::engine::{CaseReport, CaseStatus, RunReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Skip,
    Error,
}

impl Status {
    pub fn from_case(status: CaseStatus) -> Self {
        match status {
            CaseStatus::Pass => Self::Pass,
            CaseStatus::Fail => Self::Fail,
            CaseStatus::Skip => Self::Skip,
            CaseStatus::Error => Self::Error,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail | Self::Error => "FAIL",
            Self::Skip => "SKIP",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub color: bool,
    pub quiet: bool,
    pub verbose: bool,
}

fn status_label(status: Status, color: bool) -> &'static str {
    if !color {
        return status.label();
    }
    match status {
        Status::Pass => "\u{1b}[32mPASS\u{1b}[0m",
        Status::Fail | Status::Error => "\u{1b}[31mFAIL\u{1b}[0m",
        Status::Skip => "\u{1b}[33mSKIP\u{1b}[0m",
    }
}

fn contract_name(report: &RunReport, case: &CaseReport) -> String {
    format!("{}::{}", report.domain, case.contract_id)
}

fn time_block(duration_ms: u64) -> String {
    format!("[{:>7.3}s]", duration_ms as f64 / 1_000.0)
}

fn counter_block(index: usize, total: usize) -> String {
    let width = total.max(1).to_string().len();
    format!("({:>width$}/{:>width$})", index, total, width = width)
}

pub fn render(reports: &[RunReport], mode: &str, jobs: &str, fail_fast: bool, options: RenderOptions) -> String {
    let mut cases = reports
        .iter()
        .flat_map(|report| report.cases.iter().map(move |case| (report, case)))
        .collect::<Vec<_>>();
    cases.sort_by(|(left_report, left_case), (right_report, right_case)| {
        left_report
            .domain
            .cmp(&right_report.domain)
            .then_with(|| left_case.contract_id.cmp(&right_case.contract_id))
            .then_with(|| left_case.test_id.cmp(&right_case.test_id))
    });

    let total = cases.len();
    let mut lines = vec![format!(
        "contract-run: mode={mode} jobs={jobs} fail-fast={fail_fast}"
    )];
    let mut failed = Vec::new();
    let mut skipped = Vec::new();

    for (index, (report, case)) in cases.iter().enumerate() {
        let status = Status::from_case(case.status);
        let contract_name = contract_name(report, case);
        let mut line = format!(
            "{} {} {} {} {}",
            status_label(status, options.color),
            time_block(case.duration_ms),
            counter_block(index + 1, total),
            contract_name,
            case.test_id
        );
        if matches!(status, Status::Skip) {
            if let Some(note) = &case.note {
                line.push_str(&format!(" ({note})"));
            }
            skipped.push(format!("{contract_name} {}", case.test_id));
        } else if matches!(status, Status::Fail | Status::Error) {
            failed.push(format!("{contract_name} {}", case.test_id));
            if let Some(note) = &case.note {
                line.push_str(&format!(" ({note})"));
            }
        }
        if !options.quiet || !matches!(status, Status::Pass | Status::Skip) {
            lines.push(line);
        }
        if options.verbose && matches!(status, Status::Fail | Status::Error) {
            if let Some(note) = &case.note {
                lines.push(format!("  detail: {note}"));
            }
            for violation in &case.violations {
                lines.push(format!("  detail: {}", violation.message));
            }
        }
    }

    let passed = cases
        .iter()
        .filter(|(_, case)| case.status == CaseStatus::Pass)
        .count();
    let failed_count = cases
        .iter()
        .filter(|(_, case)| matches!(case.status, CaseStatus::Fail | CaseStatus::Error))
        .count();
    let skipped_count = cases
        .iter()
        .filter(|(_, case)| case.status == CaseStatus::Skip)
        .count();

    lines.push(format!(
        "contract-summary: total={total} passed={passed} failed={failed_count} skipped={skipped_count}"
    ));
    if !failed.is_empty() {
        lines.push("failed-tests:".to_string());
        lines.extend(failed);
    }
    if !skipped.is_empty() {
        lines.push("skipped-tests:".to_string());
        lines.extend(skipped);
    }
    lines.join("\n")
}
