// SPDX-License-Identifier: Apache-2.0
//! Human-readable terminal rendering.

use crate::engine;
use crate::model::RunReport as ChecksRunReport;
use crate::model::engine::RunReport;

pub fn render_contracts(report: &RunReport) -> String {
    engine::to_pretty(report)
}

pub fn render_checks(report: &ChecksRunReport) -> String {
    crate::core::render_text_summary(report)
}
