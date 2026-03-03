// SPDX-License-Identifier: Apache-2.0
//! Shared terminal line rendering for runnable status output.

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
