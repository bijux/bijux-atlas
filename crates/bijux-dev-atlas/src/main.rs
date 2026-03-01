// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]

#[path = "commands/build.rs"]
mod build_commands;
mod cli;
#[path = "commands/configs.rs"]
mod configs_commands;
#[path = "commands/control_plane.rs"]
mod control_plane_commands;
#[path = "commands/docs.rs"]
mod docs_commands;
#[path = "commands/artifacts.rs"]
mod artifacts_commands;
#[path = "commands/make.rs"]
mod make_commands;
#[cfg(test)]
#[path = "../tests/support/main_cli_parser_tests.rs"]
mod main_tests;
#[path = "commands/ops.rs"]
mod ops_commands;
#[path = "commands/ops/execution_runtime.rs"]
mod ops_execution_runtime;
#[path = "commands/ops/support.rs"]
mod ops_support;

include!("runtime_entry.rs");

#[allow(dead_code)]
fn workspace_root_resolver_anchor(arg: Option<std::path::PathBuf>) {
    let _ = bijux_dev_atlas::adapters::WorkspaceRoot::from_cli_or_cwd(arg);
}

fn main() {
    std::process::exit(run());
}
