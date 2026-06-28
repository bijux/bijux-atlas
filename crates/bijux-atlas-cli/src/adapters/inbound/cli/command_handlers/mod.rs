// SPDX-License-Identifier: Apache-2.0

mod completion;
mod environment;
mod ingest;
mod inspection;
mod metadata;
mod query;

pub(super) use self::completion::print_completion;
pub(super) use self::environment::{emit_config_paths, print_config};
pub(super) use self::ingest::run_ingest;
pub(super) use self::inspection::{inspect_dataset, inspect_db, inspect_provenance, smoke_dataset};
pub(super) use self::metadata::{
    emit_plugin_metadata, enforce_umbrella_compatibility, print_version,
};
pub(super) use self::query::{
    explain_query, explain_query_from_query_text, export_query_rows, run_query, ExplainQueryArgs,
};
