// SPDX-License-Identifier: Apache-2.0
//! Tutorials domain registration.

pub mod checks;
pub mod commands;
pub mod runtime;

use crate::domains::Domain;
use crate::model::{CommandRoute, RunnableEntry, RunnableId, RunnableKind, RunnableMode, SuiteId};
use crate::registry::RunnableRegistry;

pub struct TutorialsDomain;

pub fn plugin() -> TutorialsDomain {
    TutorialsDomain
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for TutorialsDomain {
    fn name(&self) -> &'static str {
        "tutorials"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &[
            "docs/tutorials/index.md",
            "docs/tutorials/run-with-dev-atlas.md",
        ]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        let mut runnables = registry
            .all()
            .iter()
            .filter(|entry| {
                entry.group.to_ascii_lowercase().contains("tutorial")
                    || entry.id.as_str().to_ascii_lowercase().contains("tutorial")
                    || entry
                        .commands
                        .iter()
                        .any(|command| command.contains("tutorials"))
            })
            .cloned()
            .collect::<Vec<_>>();
        if runnables.is_empty() {
            let id = RunnableId::parse("tutorials.verify");
            let suite = SuiteId::parse("tutorials");
            if let (Ok(id), Ok(suite)) = (id, suite) {
                runnables.push(RunnableEntry {
                    id,
                    suite,
                    kind: RunnableKind::Check,
                    mode: RunnableMode::Pure,
                    summary: "Validate tutorials assets and generated artifacts".to_string(),
                    owner: "docs-governance".to_string(),
                    group: "tutorials".to_string(),
                    tags: Vec::new(),
                    commands: vec!["bijux-dev-atlas tutorials verify --format json".to_string()],
                    report_ids: vec!["tutorials_verify".to_string()],
                    reports: vec!["artifacts/tutorials/verify-report.json".to_string()],
                    required_tools: vec!["bijux-dev-atlas".to_string()],
                    missing_tools_policy: "fail".to_string(),
                    effects_required: Vec::new(),
                });
            }
        }
        runnables
    }
}
