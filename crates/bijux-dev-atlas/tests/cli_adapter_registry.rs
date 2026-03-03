// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

use bijux_dev_atlas::adapters::cli::{command_inventory_markdown, describe_command, route_name};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn command_inventory_docs_match_generated_registry() {
    let expected = command_inventory_markdown();
    let public_doc = fs::read_to_string(repo_root().join("docs/cli-command-list.md"))
        .expect("read public command list");
    let internal_doc = fs::read_to_string(repo_root().join("docs/_internal/cli-command-list.md"))
        .expect("read internal command list");
    assert_eq!(public_doc.trim(), expected.trim());
    assert_eq!(internal_doc.trim(), expected.trim());
}

#[test]
fn command_routes_cover_top_level_runtime_commands() {
    for command in [
        "ops",
        "docs",
        "configs",
        "governance",
        "security",
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
