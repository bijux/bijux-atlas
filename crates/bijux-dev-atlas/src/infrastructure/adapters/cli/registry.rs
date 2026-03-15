// SPDX-License-Identifier: Apache-2.0
//! Stable command registry used by CLI adapters and generated docs.

use crate::registry::{command_routes, validate_command_routes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDescriptor {
    pub name: &'static str,
    pub domain: &'static str,
    pub purpose: &'static str,
}

pub fn command_inventory() -> Vec<CommandDescriptor> {
    let routes = command_routes();
    if let Err(err) = validate_command_routes(&routes) {
        panic!("command routes must stay valid: {err}");
    }
    routes
        .into_iter()
        .map(|route| CommandDescriptor {
            name: route.name,
            domain: route.domain,
            purpose: route.purpose,
        })
        .collect()
}

pub fn describe_command(name: &str) -> Option<CommandDescriptor> {
    command_inventory()
        .into_iter()
        .find(|entry| entry.name == name)
}

pub fn command_inventory_payload() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "kind": "cli_command_inventory",
        "commands": command_inventory().into_iter().map(|entry| serde_json::json!({
            "name": entry.name,
            "domain": entry.domain,
            "purpose": entry.purpose,
        })).collect::<Vec<_>>()
    })
}

pub fn command_inventory_markdown() -> String {
    let mut out = String::from("# CLI Command List\n\n");
    out.push_str("| Command | Domain | Purpose |\n");
    out.push_str("| --- | --- | --- |\n");
    for entry in command_inventory() {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            entry.name, entry.domain, entry.purpose
        ));
    }
    out
}
