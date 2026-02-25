// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod artifact_validation;
mod command_output_adapters;
mod commands;

use bijux_atlas_core::{
    canonical, resolve_bijux_cache_dir, resolve_bijux_config_path, sha256_hex, ConfigPathScope,
    MachineError,
};
use bijux_atlas_ingest::{diff_normalized_ids, replay_normalized_counts};
use bijux_atlas_ingest::{ingest_dataset, IngestOptions, TimestampPolicy};
use bijux_atlas_model::{
    BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, GeneIdentifierPolicy, GeneNamePolicy,
    SeqidNormalizationPolicy, ShardingPlan, StrictnessMode, TranscriptTypePolicy,
};
use bijux_atlas_query::{
    classify_query, explain_query_plan, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits,
    RegionFilter,
};
use clap::{error::ErrorKind, ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use commands::{CatalogCommand, DatasetCommand, DiffCommand, GcCommand};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode as ProcessExitCode;

const BIJUX_HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{about-with-newline}
Usage: {usage}

Options:
{options}

Commands:
{subcommands}
{after-help}";
const UMBRELLA_MIN_VERSION: &str = "0.1.0";
const UMBRELLA_MAX_EXCLUSIVE_VERSION: &str = "0.2.0";

#[derive(Parser)]
#[command(name = "bijux-atlas")]
#[command(about = "Bijux Atlas operations CLI")]
#[command(help_template = BIJUX_HELP_TEMPLATE)]
#[command(
    after_help = "Environment:\n  BIJUX_LOG_LEVEL   Log verbosity override\n  BIJUX_CACHE_DIR   Shared cache directory"
)]
struct Cli {
    #[arg(long, global = true, default_value_t = false)]
    json: bool,
    #[arg(long, global = true, default_value_t = false)]
    quiet: bool,
    #[arg(long, global = true, action = ArgAction::Count)]
    verbose: u8,
    #[arg(long, global = true, default_value_t = false)]
    trace: bool,
    #[arg(long = "bijux-plugin-metadata", default_value_t = false)]
    bijux_plugin_metadata: bool,
    #[arg(long = "print-config-paths", default_value_t = false)]
    print_config_paths: bool,
    #[arg(long = "umbrella-version")]
    umbrella_version: Option<String>,
    #[command(subcommand)]
    command: Option<AtlasCommand>,
}

#[derive(Subcommand)]
enum AtlasCommand {
    #[command(hide = true)]
    Validate {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long, default_value_t = false)]
        deep: bool,
    },
    Version,
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
    #[command(name = "config")]
    PrintConfig {
        #[arg(long, default_value_t = false)]
        canonical: bool,
    },
    Catalog {
        #[command(subcommand)]
        command: CatalogCommand,
    },
    Dataset {
        #[command(subcommand)]
        command: DatasetCommand,
    },
    Diff {
        #[command(subcommand)]
        command: DiffCommand,
    },
    Gc {
        #[command(subcommand)]
        command: GcCommand,
    },
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    Ingest {
        #[arg(long)]
        gff3: PathBuf,
        #[arg(long)]
        fasta: PathBuf,
        #[arg(long)]
        fai: PathBuf,
        #[arg(long, default_value_t = false)]
        allow_network_inputs: bool,
        #[arg(long, default_value_t = false)]
        resume: bool,
        #[arg(long)]
        output_root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
        #[arg(long, value_enum, default_value_t = StrictnessCli::Strict)]
        strictness: StrictnessCli,
        #[arg(long, value_enum, default_value_t = DuplicateGeneIdPolicyCli::Fail)]
        duplicate_gene_id_policy: DuplicateGeneIdPolicyCli,
        #[arg(long, value_enum, default_value_t = GeneIdentifierPolicyCli::Gff3Id)]
        gene_identifier_policy: GeneIdentifierPolicyCli,
        #[arg(long, default_value = "gene_id")]
        ensembl_keys: String,
        #[arg(long, default_value = "")]
        seqid_aliases: String,
        #[arg(long, default_value_t = 1)]
        max_threads: usize,
        #[arg(long, default_value_t = false)]
        report_only: bool,
        #[arg(long, default_value_t = false)]
        strict: bool,
        #[arg(long, default_value_t = false)]
        allow_overlap_gene_ids_across_contigs: bool,
        #[arg(long, default_value_t = false)]
        no_fai_check: bool,
        #[arg(long, default_value_t = false)]
        dev_auto_generate_fai: bool,
        #[arg(long, default_value_t = false)]
        fasta_scanning: bool,
        #[arg(long, default_value_t = 2000000000)]
        fasta_scan_max_bases: u64,
        #[arg(long, default_value_t = false)]
        emit_shards: bool,
        #[arg(long, default_value_t = 0)]
        shard_partitions: usize,
        #[arg(long, value_enum)]
        sharding_plan: Option<ShardingPlanCli>,
        #[arg(long, default_value_t = false)]
        emit_normalized_debug: bool,
        #[arg(long, default_value_t = false)]
        normalized_replay: bool,
        #[arg(long, default_value_t = false)]
        prod_mode: bool,
    },
    #[command(hide = true)]
    IngestVerifyInputs {
        #[arg(long)]
        gff3: PathBuf,
        #[arg(long)]
        fasta: PathBuf,
        #[arg(long)]
        fai: PathBuf,
        #[arg(long)]
        output_root: PathBuf,
        #[arg(long, default_value_t = false)]
        allow_network_inputs: bool,
        #[arg(long, default_value_t = false)]
        resume: bool,
    },
    #[command(hide = true)]
    IngestReplay {
        #[arg(long)]
        normalized: PathBuf,
    },
    #[command(hide = true)]
    IngestNormalizedDiff {
        #[arg(long)]
        base: PathBuf,
        #[arg(long)]
        target: PathBuf,
    },
    #[command(hide = true)]
    IngestValidate {
        #[arg(long)]
        qc_report: PathBuf,
        #[arg(long, default_value = "configs/ops/dataset-qc-thresholds.v1.json")]
        thresholds: PathBuf,
    },
    #[command(hide = true)]
    InspectDb {
        #[arg(long)]
        db: PathBuf,
        #[arg(long, default_value_t = 5)]
        sample_rows: usize,
    },
    #[command(hide = true)]
    ExplainQuery {
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        gene_id: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        name_prefix: Option<String>,
        #[arg(long)]
        biotype: Option<String>,
        #[arg(long)]
        region: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        allow_full_scan: bool,
    },
    #[command(hide = true)]
    Explain {
        #[arg(long)]
        db: PathBuf,
        #[arg(value_name = "QUERY")]
        query: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        allow_full_scan: bool,
    },
    #[command(hide = true)]
    Smoke {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        dataset: String,
        #[arg(long, default_value = "ops/datasets/fixtures/medium/api-list-queries.v1.json")]
        golden_queries: PathBuf,
        #[arg(long, default_value_t = false)]
        write_snapshot: bool,
        #[arg(long, default_value = "ops/datasets/fixtures/medium/api-list-responses.v1.json")]
        snapshot_out: PathBuf,
    },
    Openapi {
        #[command(subcommand)]
        command: OpenapiCommand,
    },
}

#[derive(Subcommand)]
enum OpenapiCommand {
    Generate {
        #[arg(long, default_value = "configs/openapi/v1/openapi.generated.json")]
        out: PathBuf,
    },
}

#[derive(Subcommand)]
enum PolicyCommand {
    Validate,
    Explain {
        #[arg(long, value_enum)]
        mode: Option<PolicyModeCli>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum StrictnessCli {
    Strict,
    Compat,
    Lenient,
    ReportOnly,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DuplicateGeneIdPolicyCli {
    Fail,
    Dedupe,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum GeneIdentifierPolicyCli {
    Gff3Id,
    Ensembl,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ShardingPlanCli {
    None,
    Contig,
    RegionGrid,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PolicyModeCli {
    Strict,
    Compat,
    Dev,
}

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

pub fn main_entry() -> ProcessExitCode {
    let wants_json = std::env::args().any(|arg| arg == "--json");
    match run() {
        Ok(()) => ProcessExitCode::from(bijux_atlas_core::ExitCode::Success as u8),
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
                        exit_code: bijux_atlas_core::ExitCode::Usage,
                        machine: MachineError::new(
                            "legacy_command_redirect",
                            "control-plane commands moved to bijux dev atlas",
                        )
                        .with_detail("legacy_command", &legacy_command)
                        .with_detail("redirect", redirect),
                    });
                }
                return Err(CliError {
                    exit_code: bijux_atlas_core::ExitCode::Usage,
                    machine: MachineError::new("usage_error", "invalid command line arguments")
                        .with_detail("error", &err.to_string()),
                });
            }
        },
    };
    let output_mode = OutputMode { json: cli.json };
    if cli.bijux_plugin_metadata {
        emit_plugin_metadata(output_mode.json).map_err(CliError::internal)?;
        return Ok(());
    }
    if let Some(umbrella_version) = cli.umbrella_version.as_deref() {
        enforce_umbrella_compatibility(umbrella_version)?;
    }
    if cli.print_config_paths {
        emit_config_paths(output_mode.json).map_err(CliError::internal)?;
        return Ok(());
    }

    let command = cli.command.ok_or_else(|| CliError {
        exit_code: bijux_atlas_core::ExitCode::Usage,
        machine: MachineError::new("usage_error", "missing command; see --help"),
    })?;
    let log_flags = LogFlags {
        quiet: cli.quiet,
        verbose: cli.verbose,
        trace: cli.trace,
    };

    run_atlas_command(command, log_flags, output_mode)
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
Legacy atlas namespace compatibility surface
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

#[derive(Clone, Copy)]
struct LogFlags {
    #[allow(dead_code)]
    quiet: bool,
    verbose: u8,
    #[allow(dead_code)]
    trace: bool,
}

#[derive(Clone, Copy)]
struct OutputMode {
    json: bool,
}

include!("atlas_command_dispatch.rs");
include!("atlas_command_actions.rs");
