// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

fn atlas_src_root(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/bijux-atlas/src")
}

fn dev_atlas_src_root(repo_root: &Path) -> PathBuf {
    repo_root.join("crates/bijux-dev-atlas/src")
}

#[must_use]
pub fn atlas_server_router_source(repo_root: &Path) -> PathBuf {
    atlas_src_root(repo_root).join("application/server/state/router.rs")
}

#[must_use]
pub fn atlas_request_utils_source(repo_root: &Path) -> PathBuf {
    atlas_src_root(repo_root).join("application/server/state/request_utils.rs")
}

#[must_use]
pub fn atlas_http_handlers_utilities_source(repo_root: &Path) -> PathBuf {
    atlas_src_root(repo_root).join("interfaces/http/handlers_utilities.rs")
}

#[must_use]
pub fn atlas_http_response_contract_source(repo_root: &Path) -> PathBuf {
    atlas_src_root(repo_root).join("interfaces/http/response_contract.rs")
}

#[must_use]
pub fn atlas_runtime_config_tests_source(repo_root: &Path) -> PathBuf {
    atlas_src_root(repo_root).join("application/config/runtime/tests.rs")
}

#[must_use]
pub fn dev_atlas_cli_dispatch_source(repo_root: &Path) -> PathBuf {
    dev_atlas_src_root(repo_root).join("interfaces/cli/dispatch.rs")
}

#[must_use]
pub fn dev_atlas_cli_mod_source(repo_root: &Path) -> PathBuf {
    dev_atlas_src_root(repo_root).join("interfaces/cli/mod.rs")
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
            atlas_request_utils_source(&root),
            atlas_http_handlers_utilities_source(&root),
            atlas_http_response_contract_source(&root),
            atlas_runtime_config_tests_source(&root),
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
