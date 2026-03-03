// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerfRuntimeRoute {
    pub command_name: &'static str,
    pub entrypoint: &'static str,
}

pub fn command_registry() -> Vec<PerfRuntimeRoute> {
    vec![PerfRuntimeRoute {
        command_name: "perf.run",
        entrypoint: "crate::commands/perf.rs",
    }]
}
