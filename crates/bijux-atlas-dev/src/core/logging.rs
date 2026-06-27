// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::model::CONTRACT_SCHEMA_VERSION;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogRecord {
    pub schema_version: u64,
    pub level: LogLevel,
    pub code: String,
    pub message: String,
}

impl LogRecord {
    pub fn new(level: LogLevel, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            schema_version: CONTRACT_SCHEMA_VERSION,
            level,
            code: code.into(),
            message: message.into(),
        }
    }
}

pub fn render_log(record: &LogRecord, format: LogFormat) -> Result<String, String> {
    match format {
        LogFormat::Human => Ok(format!(
            "[{}] {}: {}",
            match record.level {
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warn => "warn",
                LogLevel::Error => "error",
            },
            record.code,
            record.message
        )),
        LogFormat::Json => serde_json::to_string(record).map_err(|err| err.to_string()),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn json_log_is_stable_shape() {
        let rec = LogRecord::new(LogLevel::Info, "BOUNDARYS", "printed effect boundaries");
        let json = render_log(&rec, LogFormat::Json).expect("json");
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"level\":\"info\""));
        assert!(json.contains("\"code\":\"BOUNDARYS\""));
    }

    #[test]
    fn human_log_is_compact() {
        let rec = LogRecord::new(LogLevel::Warn, "TEST", "message");
        let text = render_log(&rec, LogFormat::Human).expect("human");
        assert_eq!(text, "[warn] TEST: message");
    }
}
