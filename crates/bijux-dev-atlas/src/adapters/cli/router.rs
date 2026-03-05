// SPDX-License-Identifier: Apache-2.0
//! Stable command-to-domain router metadata.

pub fn route_name(command_name: &str) -> Option<&'static str> {
    match command_name {
        "ops" => Some("ops"),
        "docs" => Some("docs"),
        "configs" => Some("configs"),
        "governance" => Some("governance"),
        "security" => Some("security"),
        "tutorials" => Some("tutorials"),
        "release" => Some("release"),
        "perf" => Some("perf"),
        "contract" | "suites" | "reports" | "list" | "describe" | "run" => Some("engine"),
        _ => None,
    }
}
