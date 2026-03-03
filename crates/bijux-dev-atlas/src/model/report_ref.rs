// SPDX-License-Identifier: Apache-2.0
//! Stable report path references.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportRef {
    pub report_id: String,
    pub path: String,
}
