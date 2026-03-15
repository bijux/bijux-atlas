// SPDX-License-Identifier: Apache-2.0
//! Docker domain command surface.

pub mod commands;

use crate::domains::Domain;
use crate::model::{Effect, RunnableId, RunnableKind, RunnableMode, SuiteId, Tag};
use crate::model::{CommandRoute, RunnableEntry};
use crate::registry::RunnableRegistry;

pub struct DockerDomain;

pub fn plugin() -> DockerDomain {
    DockerDomain
}

fn parse_runnable_id(value: &str) -> RunnableId {
    match RunnableId::parse(value) {
        Ok(id) => id,
        Err(err) => panic!("invalid docker route id `{value}`: {err}"),
    }
}

fn checks_suite_id() -> SuiteId {
    match SuiteId::parse("checks") {
        Ok(id) => id,
        Err(err) => panic!("invalid checks suite id: {err}"),
    }
}

fn docker_tag() -> Tag {
    match Tag::parse("docker") {
        Ok(tag) => tag,
        Err(err) => panic!("invalid docker tag: {err}"),
    }
}

pub fn routes() -> Vec<CommandRoute> {
    commands::routes()
}

impl Domain for DockerDomain {
    fn name(&self) -> &'static str {
        "docker"
    }

    fn docs_links(&self) -> &'static [&'static str] {
        &["ops/docker/README.md"]
    }

    fn required_tools(&self) -> &'static [&'static str] {
        &["docker", "bijux-dev-atlas"]
    }

    fn load_runnables(&self, registry: &RunnableRegistry) -> Vec<RunnableEntry> {
        let runnables = registry
            .all()
            .iter()
            .filter(|entry| {
                entry.group.contains("docker") || entry.id.as_str().contains("docker")
            })
            .cloned()
            .collect::<Vec<_>>();
        if !runnables.is_empty() {
            return runnables;
        }

        routes()
            .into_iter()
            .map(|route| RunnableEntry {
                id: parse_runnable_id(route.id),
                suite: checks_suite_id(),
                kind: RunnableKind::Check,
                mode: RunnableMode::Pure,
                summary: route.purpose.to_string(),
                owner: self.name().to_string(),
                group: self.name().to_string(),
                tags: vec![docker_tag()],
                commands: vec![route.name.to_string()],
                report_ids: vec![],
                reports: vec![],
                required_tools: self
                    .required_tools()
                    .iter()
                    .map(|tool| (*tool).to_string())
                    .collect(),
                missing_tools_policy: "warn".to_string(),
                effects_required: vec![Effect::FsRead],
            })
            .collect()
    }
}
