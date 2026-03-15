#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=BIJUX_ATLAS_DEV_VERSION_OVERRIDE");

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let workspace_root = bijux_versioning::workspace_root_from_manifest_dir(&manifest_dir);
    bijux_versioning::emit_git_rerun_hints(&workspace_root, |line| println!("{line}"));

    let package_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let metadata = bijux_versioning::resolve_runtime_versions(
        &workspace_root,
        &package_version,
        "BIJUX_ATLAS_DEV_VERSION_OVERRIDE",
    );

    println!(
        "cargo:rustc-env=BIJUX_ATLAS_DEV_BUILD_SEMVER_VERSION={}",
        metadata.semver_version
    );
    println!(
        "cargo:rustc-env=BIJUX_ATLAS_DEV_BUILD_DISPLAY_VERSION={}",
        metadata.display_version
    );
    println!(
        "cargo:rustc-env=BIJUX_ATLAS_DEV_BUILD_VERSION_SOURCE={}",
        metadata.source
    );
    bijux_versioning::emit_optional_env(
        "BIJUX_ATLAS_DEV_BUILD_GIT_COMMIT",
        metadata.git_commit.as_deref(),
    );
    bijux_versioning::emit_optional_env(
        "BIJUX_ATLAS_DEV_BUILD_GIT_DIRTY",
        metadata.git_dirty.map(|dirty| if dirty { "1" } else { "0" }),
    );
}
