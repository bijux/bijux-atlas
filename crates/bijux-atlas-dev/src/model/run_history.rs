// SPDX-License-Identifier: Apache-2.0
//! Stable run history models used by suite and report tooling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunHistoryEntry {
    pub suite: String,
    pub run_id: String,
    pub task_id: String,
    pub group: String,
    pub mode: String,
    pub status: String,
    pub duration_ms: u64,
    pub timestamp: String,
    pub result_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunHistory {
    pub entries: Vec<RunHistoryEntry>,
}

impl RunHistory {
    pub fn by_suite<'a>(&'a self, suite: &str) -> Vec<&'a RunHistoryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.suite == suite)
            .collect()
    }
}
