// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsRuntimeHelper {
    pub name: &'static str,
    pub source: &'static str,
}

pub fn helpers() -> Vec<DocsRuntimeHelper> {
    vec![
        DocsRuntimeHelper {
            name: "payload_builders",
            source: "crate::application/docs/runtime/payload_builders.rs",
        },
        DocsRuntimeHelper {
            name: "subprocess_support",
            source: "crate::application/docs/runtime/subprocess_support.rs",
        },
    ]
}
