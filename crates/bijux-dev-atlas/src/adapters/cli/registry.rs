// SPDX-License-Identifier: Apache-2.0
//! Stable command registry used by CLI adapters and generated docs.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDescriptor {
    pub name: &'static str,
    pub domain: &'static str,
    pub purpose: &'static str,
}

pub fn command_inventory() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            name: "ops",
            domain: "ops",
            purpose: "Run ops runtime and validation commands",
        },
        CommandDescriptor {
            name: "docs",
            domain: "docs",
            purpose: "Run docs validation and generation commands",
        },
        CommandDescriptor {
            name: "configs",
            domain: "configs",
            purpose: "Run configs validation and explanation commands",
        },
        CommandDescriptor {
            name: "governance",
            domain: "governance",
            purpose: "Inspect governance registries and policy status",
        },
        CommandDescriptor {
            name: "security",
            domain: "security",
            purpose: "Run security validation commands",
        },
        CommandDescriptor {
            name: "release",
            domain: "release",
            purpose: "Run release verification commands",
        },
        CommandDescriptor {
            name: "perf",
            domain: "perf",
            purpose: "Run performance validation commands",
        },
        CommandDescriptor {
            name: "suites",
            domain: "engine",
            purpose: "Run grouped runnable suites",
        },
        CommandDescriptor {
            name: "reports",
            domain: "engine",
            purpose: "List governed reports and validate report artifacts",
        },
        CommandDescriptor {
            name: "list",
            domain: "engine",
            purpose: "List domains, suites, and runnable ids",
        },
        CommandDescriptor {
            name: "describe",
            domain: "engine",
            purpose: "Describe one runnable without executing it",
        },
        CommandDescriptor {
            name: "run",
            domain: "engine",
            purpose: "Run one runnable by id",
        },
    ]
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
