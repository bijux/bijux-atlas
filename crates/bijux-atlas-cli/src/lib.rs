#![forbid(unsafe_code)]

mod artifact_validation;
mod command_output_adapters;
mod commands;

use bijux_atlas_core::{
    canonical, resolve_bijux_cache_dir, resolve_bijux_config_path, sha256_hex, ConfigPathScope,
    MachineError,
};
use bijux_atlas_ingest::{diff_normalized_ids, replay_normalized_counts};
use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{
    BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, GeneIdentifierPolicy, GeneNamePolicy,
    SeqidNormalizationPolicy, StrictnessMode, TranscriptTypePolicy,
};
use bijux_atlas_query::{
    classify_query, explain_query_plan, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits,
    RegionFilter,
};
use clap::{error::ErrorKind, ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use commands::{CatalogCommand, DatasetCommand};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
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
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
    Atlas {
        #[command(subcommand)]
        command: Box<AtlasCommand>,
    },
    Version,
    #[command(hide = true)]
    Serve,
}

#[derive(Subcommand)]
enum AtlasCommand {
    Serve,
    Doctor,
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
        #[arg(long, default_value_t = false)]
        emit_normalized_debug: bool,
        #[arg(long, default_value_t = false)]
        normalized_replay: bool,
        #[arg(long, default_value_t = false)]
        prod_mode: bool,
    },
    IngestReplay {
        #[arg(long)]
        normalized: PathBuf,
    },
    IngestNormalizedDiff {
        #[arg(long)]
        base: PathBuf,
        #[arg(long)]
        target: PathBuf,
    },
    IngestValidate {
        #[arg(long)]
        qc_report: PathBuf,
        #[arg(long, default_value = "configs/ops/dataset-qc-thresholds.json")]
        thresholds: PathBuf,
    },
    InspectDb {
        #[arg(long)]
        db: PathBuf,
        #[arg(long, default_value_t = 5)]
        sample_rows: usize,
    },
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
    Bench {
        #[arg(long, default_value = "query-patterns")]
        suite: String,
        #[arg(long, default_value_t = false)]
        enforce_baseline: bool,
    },
    Smoke {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        dataset: String,
        #[arg(long, default_value = "ops/fixtures/medium/golden_queries.json")]
        golden_queries: PathBuf,
        #[arg(long, default_value_t = false)]
        write_snapshot: bool,
        #[arg(long, default_value = "ops/fixtures/medium/golden_snapshot.json")]
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
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum StrictnessCli {
    Strict,
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
    fasta_scanning: bool,
    fasta_scan_max_bases: u64,
    emit_shards: bool,
    shard_partitions: usize,
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
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                print!("{err}");
                return Ok(());
            }
            _ => {
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

    match command {
        Commands::Completion { shell } => {
            print_completion(shell);
            Ok(())
        }
        Commands::Version => {
            print_version(log_flags.verbose > 0, output_mode).map_err(CliError::internal)
        }
        Commands::Atlas { command } => run_atlas_command(*command, log_flags, output_mode),
        Commands::Serve => run_serve(log_flags, output_mode).map_err(CliError::dependency),
    }
}

#[derive(Clone, Copy)]
struct LogFlags {
    quiet: bool,
    verbose: u8,
    trace: bool,
}

#[derive(Clone, Copy)]
struct OutputMode {
    json: bool,
}

fn run_atlas_command(
    command: AtlasCommand,
    log_flags: LogFlags,
    output_mode: OutputMode,
) -> Result<(), CliError> {
    match command {
        AtlasCommand::Serve => run_serve(log_flags, output_mode).map_err(CliError::dependency),
        AtlasCommand::Doctor => doctor(output_mode).map_err(CliError::internal),
        AtlasCommand::Validate {
            root,
            release,
            species,
            assembly,
            deep,
        } => artifact_validation::validate_dataset(
            root,
            &release,
            &species,
            &assembly,
            deep,
            output_mode,
        )
        .map_err(CliError::internal),
        AtlasCommand::Version => {
            print_version(log_flags.verbose > 0, output_mode).map_err(CliError::internal)
        }
        AtlasCommand::Completion { shell } => {
            print_completion(shell);
            Ok(())
        }
        AtlasCommand::PrintConfig { canonical } => {
            print_config(canonical, output_mode).map_err(CliError::internal)
        }
        AtlasCommand::Catalog { command } => match command {
            CatalogCommand::Validate { path } => {
                artifact_validation::validate_catalog(path, output_mode).map_err(CliError::internal)
            }
            CatalogCommand::Publish {
                store_root,
                catalog,
            } => artifact_validation::publish_catalog(store_root, catalog, output_mode)
                .map_err(CliError::internal),
            CatalogCommand::Rollback {
                store_root,
                release,
                species,
                assembly,
            } => artifact_validation::rollback_catalog(
                store_root,
                &release,
                &species,
                &assembly,
                output_mode,
            )
            .map_err(CliError::internal),
        },
        AtlasCommand::Dataset { command } => match command {
            DatasetCommand::Verify {
                root,
                release,
                species,
                assembly,
                deep,
            } => artifact_validation::validate_dataset(
                root,
                &release,
                &species,
                &assembly,
                deep,
                output_mode,
            )
            .map_err(CliError::internal),
            DatasetCommand::Validate {
                root,
                release,
                species,
                assembly,
            } => artifact_validation::validate_dataset(
                root,
                &release,
                &species,
                &assembly,
                false,
                output_mode,
            )
            .map_err(CliError::internal),
            DatasetCommand::Publish {
                source_root,
                store_root,
                release,
                species,
                assembly,
            } => artifact_validation::publish_dataset(
                source_root,
                store_root,
                &release,
                &species,
                &assembly,
                output_mode,
            )
            .map_err(CliError::internal),
            DatasetCommand::Pack {
                root,
                release,
                species,
                assembly,
                out,
            } => artifact_validation::pack_dataset(
                root,
                &release,
                &species,
                &assembly,
                out,
                output_mode,
            )
            .map_err(CliError::internal),
            DatasetCommand::VerifyPack { pack } => {
                artifact_validation::verify_pack(pack, output_mode).map_err(CliError::internal)
            }
        },
        AtlasCommand::Policy { command } => match command {
            PolicyCommand::Validate => {
                artifact_validation::validate_policy(output_mode).map_err(CliError::internal)
            }
        },
        AtlasCommand::Ingest {
            gff3,
            fasta,
            fai,
            output_root,
            release,
            species,
            assembly,
            strictness,
            duplicate_gene_id_policy,
            gene_identifier_policy,
            ensembl_keys,
            seqid_aliases,
            max_threads,
            report_only,
            strict,
            allow_overlap_gene_ids_across_contigs,
            no_fai_check,
            dev_auto_generate_fai,
            fasta_scanning,
            fasta_scan_max_bases,
            emit_shards,
            shard_partitions,
            emit_normalized_debug,
            normalized_replay,
            prod_mode,
        } => run_ingest(
            IngestCliArgs {
                gff3,
                fasta,
                fai,
                output_root,
                release,
                species,
                assembly,
                strictness,
                duplicate_gene_id_policy,
                gene_identifier_policy,
                ensembl_keys,
                seqid_aliases,
                max_threads,
                report_only,
                strict,
                allow_overlap_gene_ids_across_contigs,
                no_fai_check,
                dev_auto_generate_fai,
                fasta_scanning,
                fasta_scan_max_bases,
                emit_shards,
                shard_partitions,
                emit_normalized_debug,
                normalized_replay,
                prod_mode,
            },
            output_mode,
        )
        .map_err(CliError::internal),
        AtlasCommand::IngestReplay { normalized } => {
            let counts = replay_normalized_counts(&normalized)
                .map_err(|e| CliError::internal(e.to_string()))?;
            command_output_adapters::emit_ok(
                output_mode,
                json!({
                    "command":"atlas ingest-replay",
                    "status":"ok",
                    "normalized": normalized,
                    "counts": {
                        "genes": counts.genes,
                        "transcripts": counts.transcripts,
                        "exons": counts.exons
                    }
                }),
            )
            .map_err(CliError::internal)
        }
        AtlasCommand::IngestNormalizedDiff { base, target } => {
            let (removed, added) = diff_normalized_ids(&base, &target)
                .map_err(|e| CliError::internal(e.to_string()))?;
            command_output_adapters::emit_ok(
                output_mode,
                json!({
                    "command":"atlas ingest-normalized-diff",
                    "status":"ok",
                    "base": base,
                    "target": target,
                    "removed_count": removed.len(),
                    "added_count": added.len(),
                    "removed": removed,
                    "added": added
                }),
            )
            .map_err(CliError::internal)
        }
        AtlasCommand::IngestValidate {
            qc_report,
            thresholds,
        } => artifact_validation::validate_ingest_qc(qc_report, thresholds, output_mode)
            .map_err(CliError::internal),
        AtlasCommand::InspectDb { db, sample_rows } => {
            inspect_db(db, sample_rows, output_mode).map_err(CliError::internal)
        }
        AtlasCommand::ExplainQuery {
            db,
            gene_id,
            name,
            name_prefix,
            biotype,
            region,
            limit,
            allow_full_scan,
        } => explain_query(
            ExplainQueryArgs {
                db,
                gene_id,
                name,
                name_prefix,
                biotype,
                region,
                limit,
                allow_full_scan,
            },
            output_mode,
        )
        .map_err(CliError::internal),
        AtlasCommand::Explain {
            db,
            query,
            limit,
            allow_full_scan,
        } => explain_query_from_query_text(db, &query, limit, allow_full_scan, output_mode)
            .map_err(CliError::internal),
        AtlasCommand::Bench {
            suite,
            enforce_baseline,
        } => run_bench_command(&suite, enforce_baseline, output_mode).map_err(CliError::dependency),
        AtlasCommand::Smoke {
            root,
            dataset,
            golden_queries,
            write_snapshot,
            snapshot_out,
        } => smoke_dataset(
            root,
            &dataset,
            golden_queries,
            write_snapshot,
            snapshot_out,
            output_mode,
        )
        .map_err(CliError::internal),
        AtlasCommand::Openapi { command } => match command {
            OpenapiCommand::Generate { out } => {
                command_output_adapters::run_openapi_generate(out, output_mode)
            }
        }
        .map_err(CliError::dependency),
    }
}

include!("atlas_command_actions.rs");
