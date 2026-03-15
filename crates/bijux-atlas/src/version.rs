// SPDX-License-Identifier: Apache-2.0

use bijux_versioning::{RuntimeVersionEnv, RuntimeVersionInfo};

const fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

const fn runtime_env() -> RuntimeVersionEnv {
    RuntimeVersionEnv {
        name: "bijux-atlas",
        package_version: env!("CARGO_PKG_VERSION"),
        build_semver: option_env!("BIJUX_ATLAS_BUILD_SEMVER_VERSION"),
        build_display: option_env!("BIJUX_ATLAS_BUILD_DISPLAY_VERSION"),
        build_source: option_env!("BIJUX_ATLAS_BUILD_VERSION_SOURCE"),
        build_git_commit: option_env!("BIJUX_ATLAS_BUILD_GIT_COMMIT"),
        build_git_dirty: option_env!("BIJUX_ATLAS_BUILD_GIT_DIRTY"),
        build_profile: build_profile(),
    }
}

pub const fn runtime_semver() -> &'static str {
    bijux_versioning::runtime_semver(runtime_env())
}

pub const fn runtime_version() -> &'static str {
    bijux_versioning::runtime_version(runtime_env())
}

pub fn runtime_version_source() -> &'static str {
    bijux_versioning::runtime_version_source(runtime_env())
}

pub fn runtime_git_dirty() -> Option<bool> {
    bijux_versioning::runtime_git_dirty(runtime_env())
}

pub fn runtime_version_info() -> RuntimeVersionInfo {
    bijux_versioning::runtime_version_info(runtime_env())
}

pub fn runtime_version_line() -> String {
    bijux_versioning::runtime_version_line(runtime_env())
}
