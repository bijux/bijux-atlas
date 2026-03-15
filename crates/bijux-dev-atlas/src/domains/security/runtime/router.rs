// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityRuntimeRoute {
    pub command_name: &'static str,
    pub entrypoint: &'static str,
}

pub fn command_registry() -> Vec<SecurityRuntimeRoute> {
    vec![SecurityRuntimeRoute {
        command_name: "security.validate",
        entrypoint: "crate::application/security.rs",
    }]
}
