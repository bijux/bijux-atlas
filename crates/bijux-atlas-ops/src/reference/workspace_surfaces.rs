// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSurface {
    AtlasServerRouter,
    AtlasHttpRequestPolicies,
    AtlasHttpRouteSupport,
    AtlasHttpResponseContract,
    AtlasRuntimeConfigTests,
    AtlasCliBinary,
    AtlasServerBinary,
    AtlasDevCliDispatch,
    AtlasDevCliModule,
    AtlasRuntimeGeneratedRoot,
}

impl WorkspaceSurface {
    const fn relative_path(self) -> &'static str {
        match self {
            Self::AtlasServerRouter => {
                "crates/bijux-atlas-server/src/adapters/inbound/http/router.rs"
            }
            Self::AtlasHttpRequestPolicies => {
                "crates/bijux-atlas-server/src/adapters/inbound/http/request_policies/mod.rs"
            }
            Self::AtlasHttpRouteSupport => {
                "crates/bijux-atlas-server/src/adapters/inbound/http/route_support.rs"
            }
            Self::AtlasHttpResponseContract => {
                "crates/bijux-atlas-server/src/adapters/inbound/http/response_contract.rs"
            }
            Self::AtlasRuntimeConfigTests => {
                "crates/bijux-atlas-runtime/src/runtime/config/tests.rs"
            }
            Self::AtlasCliBinary => "crates/bijux-atlas-cli/src/bin/bijux-atlas.rs",
            Self::AtlasServerBinary => "crates/bijux-atlas-server/src/bin/bijux-atlas-server.rs",
            Self::AtlasDevCliDispatch => "crates/bijux-atlas-dev/src/interfaces/cli/dispatch.rs",
            Self::AtlasDevCliModule => "crates/bijux-atlas-dev/src/interfaces/cli/mod.rs",
            Self::AtlasRuntimeGeneratedRoot => "configs/generated/runtime",
        }
    }
}

#[must_use]
pub fn resolve_workspace_surface(repo_root: &Path, surface: WorkspaceSurface) -> PathBuf {
    repo_root.join(surface.relative_path())
}

#[must_use]
pub fn atlas_server_router_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasServerRouter)
}

#[must_use]
pub fn atlas_http_request_policies_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasHttpRequestPolicies)
}

#[must_use]
pub fn atlas_http_route_support_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasHttpRouteSupport)
}

#[must_use]
pub fn atlas_http_response_contract_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasHttpResponseContract)
}

#[must_use]
pub fn atlas_runtime_config_tests_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasRuntimeConfigTests)
}

#[must_use]
pub fn atlas_cli_binary_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasCliBinary)
}

#[must_use]
pub fn atlas_server_binary_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasServerBinary)
}

#[must_use]
pub fn dev_atlas_cli_dispatch_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasDevCliDispatch)
}

#[must_use]
pub fn dev_atlas_cli_mod_source(repo_root: &Path) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasDevCliModule)
}

#[must_use]
pub fn atlas_runtime_generated_artifact(repo_root: &Path, file_name: &str) -> PathBuf {
    resolve_workspace_surface(repo_root, WorkspaceSurface::AtlasRuntimeGeneratedRoot)
        .join(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn workspace_surfaces_point_to_existing_owned_paths() {
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
            resolve_workspace_surface(&root, WorkspaceSurface::AtlasRuntimeGeneratedRoot),
        ] {
            assert!(
                path.exists(),
                "missing owned workspace surface: {}",
                path.display()
            );
        }
    }
}
