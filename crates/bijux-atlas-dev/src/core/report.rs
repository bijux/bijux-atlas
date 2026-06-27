// SPDX-License-Identifier: Apache-2.0

use super::*;

pub fn render_text_summary(report: &RunReport) -> String {
    format!(
        "summary: passed={} failed={} skipped={} errors={} total={} duration_ms={}",
        report.summary.passed,
        report.summary.failed,
        report.summary.skipped,
        report.summary.errors,
        report.summary.total,
        report.timings_ms.values().sum::<u64>(),
    )
}

pub fn render_ci_summary_line(report: &RunReport) -> String {
    format!(
        "CI_SUMMARY run_id={} passed={} failed={} skipped={} errors={} total={}",
        report.run_id.as_str(),
        report.summary.passed,
        report.summary.failed,
        report.summary.skipped,
        report.summary.errors,
        report.summary.total
    )
}

pub fn render_text_with_durations(report: &RunReport, top_n: usize) -> String {
    let mut lines = vec![render_text_summary(report), render_ci_summary_line(report)];
    if top_n > 0 {
        let mut rows = report
            .results
            .iter()
            .map(|row| (row.id.as_str().to_string(), row.duration_ms))
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for (id, ms) in rows.into_iter().take(top_n) {
            lines.push(format!("duration: {id} {ms}ms"));
        }
    }
    lines.join("\n")
}

pub fn render_json(report: &RunReport) -> Result<String, String> {
    serde_json::to_string_pretty(report).map_err(|err| err.to_string())
}

pub fn render_jsonl(report: &RunReport) -> Result<String, String> {
    let mut lines = Vec::new();
    for row in &report.results {
        lines.push(serde_json::to_string(row).map_err(|err| err.to_string())?);
    }
    Ok(lines.join("\n"))
}

pub fn exit_code_for_report(report: &RunReport) -> i32 {
    if report.summary.errors > 0 {
        3
    } else if report.summary.failed > 0 {
        2
    } else if report.summary.skipped > 0 && report.summary.passed == 0 {
        4
    } else {
        0
    }
}
