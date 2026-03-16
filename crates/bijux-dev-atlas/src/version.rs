// SPDX-License-Identifier: Apache-2.0

use crate::version_support::{self, RuntimeVersionEnv, RuntimeVersionInfo};

const fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

const fn runtime_env() -> RuntimeVersionEnv {
    RuntimeVersionEnv {
        name: "bijux-dev-atlas",
        package_version: env!("CARGO_PKG_VERSION"),
        build_semver: option_env!("BIJUX_ATLAS_DEV_BUILD_SEMVER_VERSION"),
        build_display: option_env!("BIJUX_ATLAS_DEV_BUILD_DISPLAY_VERSION"),
        build_source: option_env!("BIJUX_ATLAS_DEV_BUILD_VERSION_SOURCE"),
        build_git_commit: option_env!("BIJUX_ATLAS_DEV_BUILD_GIT_COMMIT"),
        build_git_dirty: option_env!("BIJUX_ATLAS_DEV_BUILD_GIT_DIRTY"),
        build_profile: build_profile(),
    }
}

pub const fn runtime_semver() -> &'static str {
    version_support::runtime_semver(runtime_env())
}

pub const fn runtime_version() -> &'static str {
    version_support::runtime_version(runtime_env())
}

pub fn runtime_version_source() -> &'static str {
    version_support::runtime_version_source(runtime_env())
}

pub fn runtime_git_dirty() -> Option<bool> {
    version_support::runtime_git_dirty(runtime_env())
}

pub fn runtime_version_info() -> RuntimeVersionInfo {
    version_support::runtime_version_info(runtime_env())
}

pub fn runtime_version_line() -> String {
    version_support::runtime_version_line(runtime_env())
}
