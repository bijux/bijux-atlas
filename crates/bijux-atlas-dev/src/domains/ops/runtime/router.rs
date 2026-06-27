// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpsRuntimeRoute {
    pub command_name: &'static str,
    pub entrypoint: &'static str,
}

pub fn command_registry() -> Vec<OpsRuntimeRoute> {
    vec![
        OpsRuntimeRoute {
            command_name: "ops.runtime",
            entrypoint: "crate::application/ops/runtime.rs",
        },
        OpsRuntimeRoute {
            command_name: "ops.execution",
            entrypoint: "crate::application/ops/execution_runtime.rs",
        },
        OpsRuntimeRoute {
            command_name: "ops.support",
            entrypoint: "crate::application/ops/support.rs",
        },
    ]
}
