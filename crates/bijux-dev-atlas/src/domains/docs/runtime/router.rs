// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsRuntimeRoute {
    pub command_name: &'static str,
    pub entrypoint: &'static str,
}

pub fn command_registry() -> Vec<DocsRuntimeRoute> {
    vec![
        DocsRuntimeRoute {
            command_name: "docs.registry",
            entrypoint: "crate::commands/docs/runtime/docs_command_router.rs",
        },
        DocsRuntimeRoute {
            command_name: "docs.reference",
            entrypoint: "crate::commands/docs/runtime/reference_page_generators.rs",
        },
    ]
}
