// SPDX-License-Identifier: Apache-2.0

pub mod config {
    pub use bijux_atlas_runtime::runtime::config::*;

    pub(crate) fn resolve_runtime_path(path: std::path::PathBuf) -> std::path::PathBuf {
        if path.is_absolute() {
            return path;
        }
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(|path| path.parent())
            .map(std::path::Path::to_path_buf)
            .unwrap_or(manifest_dir);
        repo_root.join(path)
    }
}
