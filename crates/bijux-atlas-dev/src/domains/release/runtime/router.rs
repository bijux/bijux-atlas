// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseRuntimeRoute {
    pub command_name: &'static str,
    pub entrypoint: &'static str,
}

pub fn command_registry() -> Vec<ReleaseRuntimeRoute> {
    vec![ReleaseRuntimeRoute {
        command_name: "release.verify",
        entrypoint: "crate::application/release.rs",
    }]
}
