#![forbid(unsafe_code)]

mod artifact_validation;
mod helpers;

use bijux_atlas_core::{
    resolve_bijux_cache_dir, resolve_bijux_config_path, sha256_hex, ConfigPathScope, MachineError,
};
use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{
    BiotypePolicy, DatasetId, DuplicateGeneIdPolicy, GeneIdentifierPolicy, GeneNamePolicy,
    SeqidNormalizationPolicy, StrictnessMode, TranscriptTypePolicy,
};
use bijux_atlas_query::{
    explain_query_plan, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits, RegionFilter,
};
use clap::{error::ErrorKind, ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Generator, Shell};
use rusqlite::Connection;
use serde_json::{json, Value};
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
    #[command(hide = true)]
    Serve,
}

#[derive(Subcommand)]
enum AtlasCommand {
    Serve,
    Catalog {
        #[command(subcommand)]
        command: CatalogCommand,
    },
    Dataset {
        #[command(subcommand)]
        command: DatasetCommand,
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
    Smoke {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        dataset: String,
        #[arg(long, default_value = "fixtures/medium/golden_queries.json")]
        golden_queries: PathBuf,
        #[arg(long, default_value_t = false)]
        write_snapshot: bool,
        #[arg(long, default_value = "fixtures/medium/golden_snapshot.json")]
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
        #[arg(long, default_value = "openapi/v1/openapi.generated.json")]
        out: PathBuf,
    },
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

#[derive(Subcommand)]
enum CatalogCommand {
    Validate { path: PathBuf },
}

#[derive(Subcommand)]
enum DatasetCommand {
    Validate {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        release: String,
        #[arg(long)]
        species: String,
        #[arg(long)]
        assembly: String,
    },
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
        AtlasCommand::Catalog { command } => match command {
            CatalogCommand::Validate { path } => {
                artifact_validation::validate_catalog(path, output_mode).map_err(CliError::internal)
            }
        },
        AtlasCommand::Dataset { command } => match command {
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
                output_mode,
            )
            .map_err(CliError::internal),
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
            },
            output_mode,
        )
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
            OpenapiCommand::Generate { out } => helpers::run_openapi_generate(out, output_mode),
        }
        .map_err(CliError::dependency),
    }
}

fn print_completion<G: Generator>(generator: G) {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    generate(generator, &mut command, name, &mut std::io::stdout());
}

fn emit_config_paths(machine_json: bool) -> Result<(), String> {
    let payload = json!({
        "workspace_config": resolve_bijux_config_path(ConfigPathScope::Workspace),
        "user_config": resolve_bijux_config_path(ConfigPathScope::User),
        "cache_dir": resolve_bijux_cache_dir(),
    });
    if machine_json {
        println!(
            "{}",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

fn emit_plugin_metadata(machine_json: bool) -> Result<(), String> {
    let payload = plugin_metadata_payload();

    if machine_json {
        println!(
            "{}",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

fn run_serve(log_flags: LogFlags, output_mode: OutputMode) -> Result<(), String> {
    if log_flags.trace {
        std::env::set_var("BIJUX_LOG_LEVEL", "trace");
        std::env::set_var("RUST_LOG", "trace");
    } else if log_flags.verbose > 0 {
        std::env::set_var("BIJUX_LOG_LEVEL", "debug");
        std::env::set_var("RUST_LOG", "debug");
    } else if log_flags.quiet {
        std::env::set_var("BIJUX_LOG_LEVEL", "error");
        std::env::set_var("RUST_LOG", "error");
    }

    let current_exe =
        std::env::current_exe().map_err(|e| format!("failed to determine executable path: {e}"))?;
    let bin_dir = current_exe
        .parent()
        .ok_or_else(|| "failed to resolve executable directory".to_string())?;
    let server_bin = bin_dir.join("atlas-server");

    let status = Command::new(&server_bin).status().map_err(|e| {
        format!(
            "failed to start atlas-server at {}: {e}",
            server_bin.display()
        )
    })?;
    if status.success() {
        helpers::emit_ok(output_mode, json!({"command":"atlas serve","status":"ok"}))?;
        Ok(())
    } else {
        Err(format!("atlas-server exited with status {status}"))
    }
}

fn plugin_metadata_payload() -> Value {
    json!({
        "name": "bijux-atlas",
        "version": env!("CARGO_PKG_VERSION"),
        "compatible_umbrella": ">=0.1.0,<0.2.0",
        "build_hash": option_env!("BIJUX_BUILD_HASH").unwrap_or("dev"),
    })
}

#[derive(Debug)]
struct CliError {
    exit_code: bijux_atlas_core::ExitCode,
    machine: MachineError,
}

impl CliError {
    fn internal(message: String) -> Self {
        Self {
            exit_code: bijux_atlas_core::ExitCode::Internal,
            machine: MachineError::new("internal_error", &message),
        }
    }

    fn dependency(message: String) -> Self {
        Self {
            exit_code: bijux_atlas_core::ExitCode::DependencyFailure,
            machine: MachineError::new("dependency_failure", &message),
        }
    }
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

fn run_ingest(args: IngestCliArgs, output_mode: OutputMode) -> Result<(), String> {
    let dataset =
        DatasetId::new(&args.release, &args.species, &args.assembly).map_err(|e| e.to_string())?;

    let strictness = match args.strictness {
        StrictnessCli::Strict => StrictnessMode::Strict,
        StrictnessCli::Lenient => StrictnessMode::Lenient,
        StrictnessCli::ReportOnly => StrictnessMode::ReportOnly,
    };

    let duplicate_gene_id_policy = match args.duplicate_gene_id_policy {
        DuplicateGeneIdPolicyCli::Fail => DuplicateGeneIdPolicy::Fail,
        DuplicateGeneIdPolicyCli::Dedupe => {
            DuplicateGeneIdPolicy::DedupeKeepLexicographicallySmallest
        }
    };

    let gene_identifier_policy = match args.gene_identifier_policy {
        GeneIdentifierPolicyCli::Gff3Id => GeneIdentifierPolicy::Gff3Id,
        GeneIdentifierPolicyCli::Ensembl => GeneIdentifierPolicy::PreferEnsemblStableId {
            attribute_keys: args
                .ensembl_keys
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect(),
            fallback_to_gff3_id: !matches!(strictness, StrictnessMode::Strict),
        },
    };

    let result = ingest_dataset(&IngestOptions {
        gff3_path: args.gff3,
        fasta_path: args.fasta,
        fai_path: args.fai,
        output_root: args.output_root,
        dataset,
        strictness,
        duplicate_gene_id_policy,
        gene_identifier_policy,
        gene_name_policy: GeneNamePolicy::default(),
        biotype_policy: BiotypePolicy::default(),
        transcript_type_policy: TranscriptTypePolicy::default(),
        seqid_policy: SeqidNormalizationPolicy::from_aliases(artifact_validation::parse_alias_map(
            &args.seqid_aliases,
        )),
        max_threads: args.max_threads,
    })
    .map_err(|e| e.to_string())?;

    helpers::emit_ok(
        output_mode,
        json!({
            "command":"atlas ingest",
            "status":"ok",
            "manifest": result.manifest_path,
            "sqlite": result.sqlite_path,
            "anomaly_report": result.anomaly_report_path
        }),
    )?;
    Ok(())
}

fn inspect_db(db: PathBuf, sample_rows: usize, output_mode: OutputMode) -> Result<(), String> {
    let conn = Connection::open(db).map_err(|e| e.to_string())?;
    let schema_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut idx_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|e| e.to_string())?;
    let indexes = idx_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let sql = format!(
        "SELECT gene_id, name, seqid, start, end FROM gene_summary ORDER BY seqid, start, gene_id LIMIT {}",
        sample_rows
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    helpers::emit_ok(
        output_mode,
        json!({
            "command":"atlas inspect-db",
            "schema_version": schema_version,
            "indexes": indexes,
            "gene_count": count,
            "sample_rows": rows
        }),
    )?;
    Ok(())
}

struct ExplainQueryArgs {
    db: PathBuf,
    gene_id: Option<String>,
    name: Option<String>,
    name_prefix: Option<String>,
    biotype: Option<String>,
    region: Option<String>,
    limit: usize,
    allow_full_scan: bool,
}

fn explain_query(args: ExplainQueryArgs, output_mode: OutputMode) -> Result<(), String> {
    let conn = Connection::open(args.db).map_err(|e| e.to_string())?;
    let region_filter = if let Some(raw) = args.region {
        let (seqid, span) = raw
            .split_once(':')
            .ok_or_else(|| "region must be seqid:start-end".to_string())?;
        let (start, end) = span
            .split_once('-')
            .ok_or_else(|| "region must be seqid:start-end".to_string())?;
        Some(RegionFilter {
            seqid: seqid.to_string(),
            start: start.parse::<u64>().map_err(|e| e.to_string())?,
            end: end.parse::<u64>().map_err(|e| e.to_string())?,
        })
    } else {
        None
    };

    let req = GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id: args.gene_id,
            name: args.name,
            name_prefix: args.name_prefix,
            biotype: args.biotype,
            region: region_filter,
        },
        limit: args.limit,
        cursor: None,
        allow_full_scan: args.allow_full_scan,
    };
    let lines = explain_query_plan(&conn, &req, &QueryLimits::default(), b"atlas-cli")
        .map_err(|e| e.to_string())?;
    helpers::emit_ok(
        output_mode,
        json!({
            "command":"atlas explain-query",
            "plan": lines
        }),
    )?;
    Ok(())
}

fn smoke_dataset(
    root: PathBuf,
    dataset: &str,
    golden_queries: PathBuf,
    write_snapshot: bool,
    snapshot_out: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let (release, species, assembly) = helpers::parse_dataset_id(dataset)?;
    let id = DatasetId::new(&release, &species, &assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::artifact_paths(&root, &id);
    let conn = Connection::open(&paths.sqlite).map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count <= 0 {
        return Err("smoke failed: gene_summary is empty".to_string());
    }

    let raw = fs::read_to_string(golden_queries).map_err(|e| e.to_string())?;
    let queries: Vec<Value> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let mut out = Vec::new();

    for q in queries {
        let name = q
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "golden query missing name".to_string())?;
        let body = q
            .get("query")
            .ok_or_else(|| "golden query missing query object".to_string())?;
        let req = helpers::query_request_from_json(body)?;
        let resp = bijux_atlas_query::query_genes(&conn, &req, &QueryLimits::default(), b"smoke")
            .map_err(|e| e.to_string())?;
        if resp.rows.is_empty() && name == "by_gene_id" {
            return Err("smoke failed: by_gene_id returned zero rows".to_string());
        }
        out.push(serde_json::json!({
            "name": name,
            "row_count": resp.rows.len(),
            "next_cursor": resp.next_cursor,
        }));
    }

    if write_snapshot {
        let payload = serde_json::json!({ "dataset": dataset, "queries": out });
        fs::write(
            snapshot_out,
            serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }

    helpers::emit_ok(
        output_mode,
        json!({
            "command":"atlas smoke",
            "status":"ok",
            "dataset": dataset,
            "queries": out.len()
        }),
    )?;
    Ok(())
}
