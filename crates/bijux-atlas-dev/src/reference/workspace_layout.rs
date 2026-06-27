// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn atlas_src_root(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/bijux-atlas-runtime/src")
}

fn atlas_server_http_src_root(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/bijux-atlas-server/src/adapters/inbound/http")
}

fn atlas_cli_crate_root(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/bijux-atlas-cli")
}

fn atlas_server_crate_root(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/bijux-atlas-server")
}

fn dev_atlas_src_root(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/bijux-atlas-dev/src")
}

#[must_use]
pub fn atlas_server_router_source(repo_root: &Path) -> PathBuf {
    atlas_server_http_src_root(repo_root).join("router.rs")
}

#[must_use]
pub fn atlas_http_request_policies_source(repo_root: &Path) -> PathBuf {
    atlas_server_http_src_root(repo_root).join("request_policies.rs")
}

#[must_use]
pub fn atlas_http_handlers_utilities_source(repo_root: &Path) -> PathBuf {
    atlas_server_http_src_root(repo_root).join("handlers_utilities.rs")
}

#[must_use]
pub fn atlas_http_response_contract_source(repo_root: &Path) -> PathBuf {
    atlas_server_http_src_root(repo_root).join("response_contract.rs")
}

#[must_use]
pub fn atlas_runtime_config_tests_source(repo_root: &Path) -> PathBuf {
    atlas_src_root(repo_root).join("runtime/config/tests.rs")
}

#[must_use]
pub fn atlas_cli_binary_source(repo_root: &Path) -> PathBuf {
    atlas_cli_crate_root(repo_root).join("src/bin/bijux-atlas.rs")
}

#[must_use]
pub fn atlas_server_binary_source(repo_root: &Path) -> PathBuf {
    atlas_server_crate_root(repo_root).join("src/bin/bijux-atlas-server.rs")
}

#[must_use]
pub fn dev_atlas_cli_dispatch_source(repo_root: &Path) -> PathBuf {
    dev_atlas_src_root(repo_root).join("interfaces/cli/dispatch.rs")
}

#[must_use]
pub fn dev_atlas_cli_mod_source(repo_root: &Path) -> PathBuf {
    dev_atlas_src_root(repo_root).join("interfaces/cli/mod.rs")
}

#[must_use]
pub fn atlas_runtime_generated_artifact(repo_root: &Path, file_name: &str) -> PathBuf {
    repo_root
        .join("configs")
        .join("generated")
        .join("runtime")
        .join(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn canonical_workspace_paths_exist() {
        let root = repo_root();
        for path in [
            atlas_server_router_source(&root),
            atlas_http_request_policies_source(&root),
            atlas_http_handlers_utilities_source(&root),
            atlas_http_response_contract_source(&root),
            atlas_runtime_config_tests_source(&root),
            atlas_cli_binary_source(&root),
            atlas_server_binary_source(&root),
            dev_atlas_cli_dispatch_source(&root),
            dev_atlas_cli_mod_source(&root),
        ] {
            assert!(
                path.exists(),
                "missing canonical workspace path: {}",
                path.display()
            );
        }
    }
}
