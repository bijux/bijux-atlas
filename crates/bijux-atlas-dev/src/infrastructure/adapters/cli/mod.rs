// SPDX-License-Identifier: Apache-2.0
//! Canonical CLI adapter metadata.

mod registry;
mod router;

pub use registry::{
    command_inventory_markdown, command_inventory_payload, describe_command, CommandDescriptor,
};
pub use router::route_name;
