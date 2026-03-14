// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::adapters::cli::{describe_command, route_name};

#[test]
fn command_routes_cover_top_level_runtime_commands() {
    for command in [
        "ops",
        "docs",
        "configs",
        "governance",
        "security",
        "tutorials",
        "release",
        "perf",
        "contract",
        "reports",
    ] {
        assert_eq!(
            describe_command(command).expect("command metadata").domain,
            route_name(command).expect("route"),
        );
    }
}
