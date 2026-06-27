// SPDX-License-Identifier: Apache-2.0
//! Stable command registry used by CLI adapters and generated docs.

use crate::registry::{command_routes, validate_command_routes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDescriptor {
    pub name: &'static str,
    pub domain: &'static str,
    pub purpose: &'static str,
}

fn validated_command_routes() -> Result<Vec<crate::model::CommandRoute>, String> {
    let routes = command_routes();
    validate_command_routes(&routes)?;
    Ok(routes)
}

pub fn command_inventory() -> Result<Vec<CommandDescriptor>, String> {
    Ok(validated_command_routes()?
        .into_iter()
        .map(|route| CommandDescriptor {
            name: route.name,
            domain: route.domain,
            purpose: route.purpose,
        })
        .collect())
}

pub fn describe_command(name: &str) -> Result<Option<CommandDescriptor>, String> {
    Ok(command_inventory()?
        .into_iter()
        .find(|entry| entry.name == name))
}

pub fn command_inventory_payload() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "cli_command_inventory",
        "commands": command_inventory()?.into_iter().map(|entry| serde_json::json!({
            "name": entry.name,
            "domain": entry.domain,
            "purpose": entry.purpose,
        })).collect::<Vec<_>>()
    }))
}

pub fn command_inventory_markdown() -> Result<String, String> {
    let mut out = String::from("# CLI Command List\n\n");
    out.push_str("| Command | Domain | Purpose |\n");
    out.push_str("| --- | --- | --- |\n");
    for entry in command_inventory()? {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            entry.name, entry.domain, entry.purpose
        ));
    }
    Ok(out)
}
