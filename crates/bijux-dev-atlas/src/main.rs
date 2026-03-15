// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]

#[path = "application/api.rs"]
mod api_commands;
#[path = "application/artifacts.rs"]
mod artifacts_commands;
#[path = "application/audit.rs"]
mod audit_commands;
mod bootstrap;
#[path = "application/build.rs"]
mod build_commands;
mod interfaces;
#[path = "application/data.rs"]
mod commands_data;
#[path = "application/configs.rs"]
mod configs_commands;
#[path = "application/control_plane.rs"]
mod control_plane_commands;
#[path = "application/docs.rs"]
mod docs_commands;
#[path = "application/drift.rs"]
mod drift_commands;
#[path = "application/governance.rs"]
mod governance_commands;
#[path = "application/invariants.rs"]
mod invariants_commands;
#[path = "application/load.rs"]
mod load_commands;
#[cfg(test)]
#[path = "../tests/support/main_cli_parser_tests.rs"]
mod main_tests;
#[path = "application/makes.rs"]
mod makes_commands;
#[path = "application/migrations.rs"]
mod migrations_commands;
#[path = "application/observe.rs"]
mod observe_commands;
#[path = "application/ops.rs"]
mod ops_commands;
#[path = "application/ops/execution_runtime.rs"]
mod ops_execution_runtime;
#[path = "application/ops/support.rs"]
mod ops_support;
#[path = "application/perf.rs"]
mod perf_commands;
#[path = "application/release.rs"]
mod release_commands;
#[path = "application/reproduce.rs"]
mod reproduce_commands;
#[path = "application/runtime.rs"]
mod runtime_commands;
#[path = "application/security.rs"]
mod security_commands;
#[path = "application/suites.rs"]
mod suites_commands;
#[path = "application/system.rs"]
mod system_commands;
#[path = "application/tutorials.rs"]
mod tutorials_commands;

pub(crate) use self::bootstrap::*;
pub(crate) use self::interfaces::cli;
pub(crate) use bijux_dev_atlas::reference;

#[allow(dead_code)]
fn workspace_root_resolver_anchor(arg: Option<std::path::PathBuf>) {
    let _ = bijux_dev_atlas::runtime::WorkspaceRoot::from_cli_or_cwd(arg);
}

fn main() {
    std::process::exit(run());
}
