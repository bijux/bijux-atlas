// SPDX-License-Identifier: Apache-2.0

mod actions;
mod args;
mod commands;
mod dispatch;
mod ingest_inputs;
mod operations;
pub(crate) mod output;

use crate::contracts::errors::{ConfigPathScope, ExitCode, MachineError};
use crate::domain::canonical;
use crate::ingest::{
    diff_normalized_ids, ingest_dataset, replay_normalized_counts, IngestOptions, TimestampPolicy,
};
use crate::runtime::config::{resolve_bijux_cache_dir, resolve_bijux_config_path};
use crate::query::{
    classify_query, explain_query_plan, BiotypePolicy, DuplicateGeneIdPolicy, GeneFields,
    GeneFilter, GeneNamePolicy, GeneQueryRequest, QueryLimits, RegionFilter,
    SeqidNormalizationPolicy, TranscriptTypePolicy,
};
use crate::domain::dataset::{DatasetId, ShardingPlan};
use crate::domain::policy::{GeneIdentifierPolicy, StrictnessMode};
use clap::{error::ErrorKind, CommandFactory, Parser};
use clap_complete::{generate, Generator};
use commands::{CatalogCommand, DatasetCommand, DiffCommand, GcCommand};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode as ProcessExitCode;

use self::args::{
    AtlasCommand, Cli, DuplicateGeneIdPolicyCli, GeneIdentifierPolicyCli, OpenapiCommand,
    PolicyCommand, PolicyModeCli, ShardingPlanCli, StrictnessCli,
};

pub(crate) const BIJUX_HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{about-with-newline}
Usage: {usage}

Options:
{options}

Commands:
{subcommands}
{after-help}";
const UMBRELLA_MIN_VERSION: &str = "0.3.0";
const UMBRELLA_MAX_EXCLUSIVE_VERSION: &str = "0.4.0";

struct IngestCliArgs {
    gff3: PathBuf,
    fasta: PathBuf,
    fai: PathBuf,
    output_root: PathBuf,
    release: String,
    species: String,
    assembly: String,
    strictness: StrictnessCli,
    duplicate_gene_id_policy: DuplicateGeneIdPolicyCli,
    gene_identifier_policy: GeneIdentifierPolicyCli,
    ensembl_keys: String,
    seqid_aliases: String,
    max_threads: usize,
    report_only: bool,
    strict: bool,
    allow_overlap_gene_ids_across_contigs: bool,
    no_fai_check: bool,
    dev_auto_generate_fai: bool,
    allow_network_inputs: bool,
    resume: bool,
    fasta_scanning: bool,
    fasta_scan_max_bases: u64,
    emit_shards: bool,
    shard_partitions: usize,
    sharding_plan: Option<ShardingPlanCli>,
    emit_normalized_debug: bool,
    normalized_replay: bool,
    prod_mode: bool,
}

#[derive(Debug)]
struct CliError {
    exit_code: ExitCode,
    machine: MachineError,
}

impl CliError {
    fn internal(message: String) -> Self {
        Self {
            exit_code: ExitCode::Internal,
            machine: MachineError::new("internal_error", &message),
        }
    }
}

#[derive(Clone, Copy)]
struct LogFlags {
    #[allow(dead_code)]
    quiet: bool,
    verbose: u8,
    #[allow(dead_code)]
    trace: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct OutputMode {
    pub(crate) json: bool,
}

pub fn main_entry() -> ProcessExitCode {
    let wants_json = std::env::args().any(|arg| arg == "--json");
    match run() {
        Ok(()) => ProcessExitCode::from(ExitCode::Success as u8),
        Err(err) => {
            emit_error(&err, wants_json);
            ProcessExitCode::from(err.exit_code as u8)
        }
    }
}

fn run() -> Result<(), CliError> {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let (parse_args, used_legacy_namespace) = normalize_legacy_namespace(&raw_args);
    if used_legacy_namespace
        && parse_args.len() == 2
        && (parse_args[1] == "--help" || parse_args[1] == "-h")
    {
        print_legacy_atlas_help();
        return Ok(());
    }
    let cli = match Cli::try_parse_from(parse_args) {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                print!("{err}");
                return Ok(());
            }
            _ => {
                if let Some(redirect) = legacy_control_plane_redirect(&raw_args) {
                    let legacy_command = raw_args.join(" ");
                    return Err(CliError {
                        exit_code: ExitCode::Usage,
                        machine: MachineError::new(
                            "legacy_command_redirect",
                            "control-plane commands moved to bijux dev atlas",
                        )
                        .with_detail("legacy_command", &legacy_command)
                        .with_detail("redirect", redirect),
                    });
                }
                return Err(CliError {
                    exit_code: ExitCode::Usage,
                    machine: MachineError::new("usage_error", "invalid command line arguments")
                        .with_detail("error", &err.to_string()),
                });
            }
        },
    };
    let output_mode = OutputMode { json: cli.json };
    if cli.bijux_plugin_metadata {
        actions::emit_plugin_metadata(output_mode.json).map_err(CliError::internal)?;
        return Ok(());
    }
    if let Some(umbrella_version) = cli.umbrella_version.as_deref() {
        actions::enforce_umbrella_compatibility(umbrella_version)?;
    }
    if cli.print_config_paths {
        actions::emit_config_paths(output_mode.json).map_err(CliError::internal)?;
        return Ok(());
    }

    let command = cli.command.ok_or_else(|| CliError {
        exit_code: ExitCode::Usage,
        machine: MachineError::new("usage_error", "missing command; see --help"),
    })?;
    let log_flags = LogFlags {
        quiet: cli.quiet,
        verbose: cli.verbose,
        trace: cli.trace,
    };

    dispatch::run_atlas_command(command, log_flags, output_mode)
}

fn legacy_control_plane_redirect(args: &[String]) -> Option<&'static str> {
    if args.is_empty() {
        return None;
    }
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--json"
            | "--quiet"
            | "--trace"
            | "--verbose"
            | "--bijux-plugin-metadata"
            | "--print-config-paths" => i += 1,
            "--umbrella-version" => i += 2,
            token if token.starts_with('-') => i += 1,
            _ => break,
        }
    }
    if i >= args.len() {
        return None;
    }
    if args[i] == "atlas" {
        i += 1;
    }
    if i >= args.len() {
        return None;
    }
    let subcommand = args[i].as_str();
    if subcommand == "dev-atlas" {
        return Some("bijux dev atlas <command>");
    }
    if subcommand == "doctor" || subcommand == "check" || subcommand == "checks" {
        return Some("bijux dev atlas run <selector>");
    }
    None
}

fn normalize_legacy_namespace(args: &[String]) -> (Vec<String>, bool) {
    let mut atlas_index = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--json"
            | "--quiet"
            | "--trace"
            | "--verbose"
            | "--bijux-plugin-metadata"
            | "--print-config-paths" => {
                i += 1;
            }
            "--umbrella-version" => {
                i += 2;
            }
            token if token.starts_with('-') => {
                i += 1;
            }
            "atlas" => {
                atlas_index = Some(i);
                break;
            }
            _ => break,
        }
    }

    let mut normalized = vec!["bijux-atlas".to_string()];
    let mut used = false;
    for (idx, arg) in args.iter().enumerate() {
        if Some(idx) == atlas_index {
            used = true;
            continue;
        }
        normalized.push(arg.clone());
    }
    normalize_legacy_command_aliases(&mut normalized);
    (normalized, used)
}

fn normalize_legacy_command_aliases(args: &mut [String]) {
    if let Some(first_positional) = args
        .iter_mut()
        .skip(1)
        .find(|token| !token.starts_with('-'))
    {
        if first_positional == "print-config" {
            *first_positional = "config".to_string();
        }
    }
}

fn print_legacy_atlas_help() {
    const LEGACY_ATLAS_HELP: &str = "\
Legacy atlas namespace command mapping
Commands:
  ingest
  serve
  catalog
  dataset
  openapi
  completion
  version
  validate
  explain
  diff
  gc
  bench
  policy
  print-config
  smoke
  inspect-db
  ingest-verify-inputs
  ingest-validate
  ingest-replay
  ingest-normalized-diff
";
    print!("{LEGACY_ATLAS_HELP}");
}

fn emit_error(error: &CliError, machine_json: bool) {
    if machine_json {
        match serde_json::to_string(&error.machine) {
            Ok(payload) => eprintln!("{payload}"),
            Err(_) => eprintln!(
                "{{\"code\":\"internal_error\",\"message\":\"failed to encode structured error\",\"details\":{{}}}}"
            ),
        }
    } else {
        eprintln!("{}", error.machine.message);
    }
}
