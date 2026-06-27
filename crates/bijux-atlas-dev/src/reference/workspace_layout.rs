// SPDX-License-Identifier: Apache-2.0

pub use bijux_atlas_ops::reference::workspace_surface_registry::{
    atlas_cli_binary_source, atlas_http_request_policies_source,
    atlas_http_response_contract_source, atlas_http_route_support_source,
    atlas_runtime_config_tests_source, atlas_runtime_generated_artifact,
    atlas_server_binary_source, atlas_server_router_source, dev_atlas_cli_dispatch_source,
    dev_atlas_cli_mod_source,
};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn delegated_workspace_surfaces_exist() {
        let root = repo_root();
        for path in [
            atlas_server_router_source(&root),
            atlas_http_request_policies_source(&root),
            atlas_http_route_support_source(&root),
            atlas_http_response_contract_source(&root),
            atlas_runtime_config_tests_source(&root),
            atlas_cli_binary_source(&root),
            atlas_server_binary_source(&root),
            dev_atlas_cli_dispatch_source(&root),
            dev_atlas_cli_mod_source(&root),
        ] {
            assert!(
                path.exists(),
                "missing delegated workspace surface: {}",
                path.display()
            );
        }
    }
}
