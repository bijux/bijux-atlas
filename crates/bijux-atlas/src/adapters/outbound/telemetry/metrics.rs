// SPDX-License-Identifier: Apache-2.0

pub(crate) const METRICS_NAMESPACE: &str = "atlas";

#[allow(dead_code)]
pub(crate) fn is_allowed_metric_name(name: &str) -> bool {
    name.starts_with("atlas_") || name.starts_with("bijux_") || name.starts_with("http_")
}
