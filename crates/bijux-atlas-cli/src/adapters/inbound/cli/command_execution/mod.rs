// SPDX-License-Identifier: Apache-2.0

mod completion;
mod environment;
mod metadata;

pub(super) use self::completion::print_completion;
pub(super) use self::environment::{emit_config_paths, print_config};
pub(super) use self::metadata::{
    emit_plugin_metadata, enforce_umbrella_compatibility, print_version,
};
