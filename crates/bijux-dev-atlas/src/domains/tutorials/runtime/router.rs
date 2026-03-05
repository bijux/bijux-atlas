// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialsRuntimeRoute {
    pub command_name: &'static str,
    pub entrypoint: &'static str,
}

pub fn command_registry() -> Vec<TutorialsRuntimeRoute> {
    vec![TutorialsRuntimeRoute {
        command_name: "tutorials",
        entrypoint: "crate::commands/tutorials.rs",
    }]
}

