// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpsSupportModule {
    pub name: &'static str,
    pub source: &'static str,
}

pub fn support_modules() -> Vec<OpsSupportModule> {
    vec![
        OpsSupportModule {
            name: "domain_support",
            source: "crate::commands/ops/support/domain_support.rs",
        },
        OpsSupportModule {
            name: "manifests",
            source: "crate::commands/ops/support/manifests.rs",
        },
        OpsSupportModule {
            name: "reports",
            source: "crate::commands/ops/support/reports.rs",
        },
        OpsSupportModule {
            name: "tools",
            source: "crate::commands/ops/support/tools.rs",
        },
    ]
}
