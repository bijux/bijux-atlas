// SPDX-License-Identifier: Apache-2.0

pub const fn runtime_semver() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub const fn runtime_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
