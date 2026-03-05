// SPDX-License-Identifier: Apache-2.0
//! Canonical route registry built from domain-owned route surfaces.

use std::collections::BTreeSet;

use crate::domains;
use crate::model::CommandRoute;

pub fn command_routes() -> Vec<CommandRoute> {
    let mut routes = Vec::new();
    routes.extend(domains::ops::routes());
    routes.extend(domains::docs::routes());
    routes.extend(domains::configs::routes());
    routes.extend(domains::governance::routes());
    routes.extend(domains::security::routes());
    routes.extend(domains::tutorials::routes());
    routes.extend(domains::release::routes());
    routes.extend(domains::perf::routes());
    routes.extend(domains::docker::routes());
    routes.extend(engine_routes());
    routes.sort_by(|a, b| a.name.cmp(b.name).then_with(|| a.id.cmp(b.id)));
    routes
}

pub fn validate_command_routes(routes: &[CommandRoute]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for route in routes {
        if route.id.trim().is_empty() {
            return Err("route id cannot be empty".to_string());
        }
        if route.name.trim().is_empty() {
            return Err(format!(
                "route `{}` must declare a non-empty name",
                route.id
            ));
        }
        if route.purpose.trim().is_empty() {
            return Err(format!(
                "route `{}` must declare a non-empty purpose",
                route.id
            ));
        }
        if !ids.insert(route.id) {
            return Err(format!("duplicate route id `{}`", route.id));
        }
        if !names.insert(route.name) {
            return Err(format!("duplicate route name `{}`", route.name));
        }
    }
    Ok(())
}

fn engine_routes() -> [CommandRoute; 7] {
    [
        CommandRoute::new(
            "contract",
            "contract",
            "engine",
            "Run governed contract lanes and introspection surfaces",
        ),
        CommandRoute::new(
            "checks",
            "checks",
            "engine",
            "List and explain registry-backed checks surfaces",
        ),
        CommandRoute::new(
            "reports",
            "reports",
            "engine",
            "List governed reports and validate report artifacts",
        ),
        CommandRoute::new("suites", "suites", "engine", "Run grouped runnable suites"),
        CommandRoute::new(
            "list",
            "list",
            "engine",
            "List domains, suites, and runnable ids",
        ),
        CommandRoute::new(
            "describe",
            "describe",
            "engine",
            "Describe one runnable without executing it",
        ),
        CommandRoute::new("run", "run", "engine", "Run one runnable by id"),
    ]
}
