// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn main_uses_adapter_workspace_root_resolver() {
    let main_rs = fs::read_to_string(crate_root().join("src/main.rs")).unwrap_or_default();
    assert!(
        main_rs.contains("WorkspaceRoot::from_cli_or_cwd"),
        "src/main.rs must use adapters::WorkspaceRoot as the single workspace root resolver entrypoint"
    );
    assert!(
        !main_rs.contains("fn discover_repo_root("),
        "src/main.rs must not define a local repo-root discovery function"
    );
}
