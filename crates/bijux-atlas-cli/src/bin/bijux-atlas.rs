// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

fn main() -> std::process::ExitCode {
    bijux_atlas_cli::adapters::inbound::cli::main_entry()
}
