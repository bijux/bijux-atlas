// SPDX-License-Identifier: Apache-2.0
//! Shared terminal line rendering for runnable status output.

use crate::model::{CheckStatus, RunReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Pass,
    Fail,
    Skip,
    Error,
}

impl LineStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
            Self::Error => "ERROR",
        }
    }

    pub fn render(self, color: bool) -> &'static str {
        if !color {
            return self.label();
        }
        match self {
            Self::Pass => "\u{1b}[32mPASS\u{1b}[0m",
            Self::Fail | Self::Error => "\u{1b}[31mFAIL\u{1b}[0m",
            Self::Skip => "\u{1b}[33mSKIP\u{1b}[0m",
        }
    }
}

pub fn render_status_line(
    style: LineStyle,
    color: bool,
    duration_ms: u64,
    index: usize,
    total: usize,
    subject: &str,
    test_name: &str,
) -> String {
    format!(
        "{} {} {} {} {}",
        style.render(color),
        time_block(duration_ms),
        counter_block(index, total),
        subject,
        test_name
    )
}

pub fn time_block(duration_ms: u64) -> String {
    format!("[{:>7.3}s]", duration_ms as f64 / 1_000.0)
}

pub fn counter_block(index: usize, total: usize) -> String {
    let width = total.max(1).to_string().len();
    format!("({:>width$}/{:>width$})", index, total, width = width)
}

fn line_style_for_check(status: CheckStatus) -> LineStyle {
    match status {
        CheckStatus::Pass => LineStyle::Pass,
        CheckStatus::Fail => LineStyle::Fail,
        CheckStatus::Skip => LineStyle::Skip,
        CheckStatus::Error => LineStyle::Error,
    }
}

pub fn render_check_run_report(report: &RunReport, color: bool) -> String {
    let mut results = report.results.iter().collect::<Vec<_>>();
    results.sort_by(|left, right| left.id.cmp(&right.id));

    let total = results.len();
    let mut lines = vec![format!(
        "check-run: run_id={} total={} fail-fast=false",
        report.run_id.as_str(),
        total
    )];
    let mut failed = Vec::new();
    let mut skipped = Vec::new();

    for (index, result) in results.iter().enumerate() {
        let subject = format!("checks::{}", result.id);
        let mut line = render_status_line(
            line_style_for_check(result.status),
            color,
            result.duration_ms,
            index + 1,
            total,
            &subject,
            "main",
        );
        match result.status {
            CheckStatus::Skip => {
                if let Some(reason) = &result.skip_reason {
                    line.push_str(&format!(" ({reason})"));
                }
                skipped.push(format!("{subject} main"));
            }
            CheckStatus::Fail | CheckStatus::Error => {
                if let Some(violation) = result.violations.first() {
                    line.push_str(&format!(" ({})", violation.message));
                }
                failed.push(format!("{subject} main"));
            }
            CheckStatus::Pass => {}
        }
        lines.push(line);
    }

    lines.push(format!(
        "check-summary: total={} passed={} failed={} skipped={} errors={}",
        report.summary.total,
        report.summary.passed,
        report.summary.failed,
        report.summary.skipped,
        report.summary.errors
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
