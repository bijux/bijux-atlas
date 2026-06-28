// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TraceContext {
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}
