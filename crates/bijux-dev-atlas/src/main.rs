// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]

#[path = "commands/api.rs"]
mod api_commands;
mod app;
#[path = "commands/artifacts.rs"]
mod artifacts_commands;
#[path = "commands/audit.rs"]
mod audit_commands;
#[path = "commands/build.rs"]
mod build_commands;
mod cli;
#[path = "commands/data.rs"]
mod commands_data;
#[path = "commands/configs.rs"]
mod configs_commands;
#[path = "commands/control_plane.rs"]
mod control_plane_commands;
#[path = "commands/docs.rs"]
mod docs_commands;
#[path = "commands/drift.rs"]
mod drift_commands;
#[path = "commands/governance.rs"]
mod governance_commands;
#[path = "commands/invariants.rs"]
mod invariants_commands;
#[path = "commands/load.rs"]
mod load_commands;
#[cfg(test)]
#[path = "../tests/support/main_cli_parser_tests.rs"]
mod main_tests;
#[path = "commands/make.rs"]
mod make_commands;
#[path = "commands/observe.rs"]
mod observe_commands;
#[path = "commands/ops.rs"]
mod ops_commands;
#[path = "commands/ops/execution_runtime.rs"]
mod ops_execution_runtime;
#[path = "commands/ops/support.rs"]
mod ops_support;
#[path = "commands/perf.rs"]
mod perf_commands;
#[path = "commands/release.rs"]
mod release_commands;
#[path = "commands/reproduce.rs"]
mod reproduce_commands;
#[path = "commands/security.rs"]
mod security_commands;
#[path = "commands/suites.rs"]
mod suites_commands;
#[path = "commands/system.rs"]
mod system_commands;

include!("runtime_entry.rs");

#[allow(dead_code)]
fn workspace_root_resolver_anchor(arg: Option<std::path::PathBuf>) {
    let _ = bijux_dev_atlas::runtime::WorkspaceRoot::from_cli_or_cwd(arg);
}

fn main() {
    std::process::exit(app::run());
}
