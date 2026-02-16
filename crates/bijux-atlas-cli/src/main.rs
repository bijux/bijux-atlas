#![forbid(unsafe_code)]

use bijux_atlas_core::sha256_hex;
use bijux_atlas_ingest::{ingest_dataset, IngestOptions};
use bijux_atlas_model::{
    ArtifactManifest, BiotypePolicy, Catalog, DatasetId, DuplicateGeneIdPolicy,
    GeneIdentifierPolicy, GeneNamePolicy, SeqidNormalizationPolicy, StrictnessMode,
    TranscriptTypePolicy,
};
use bijux_atlas_query::{
    explain_query_plan, GeneFields, GeneFilter, GeneQueryRequest, QueryLimits, RegionFilter,
};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode as ProcessExitCode;

#[derive(Parser)]
#[command(name = "bijux-atlas")]
#[command(about = "Bijux Atlas operations CLI")]
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
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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
}

fn main() -> ProcessExitCode {
    match run() {
        Ok(()) => ProcessExitCode::from(bijux_atlas_core::ExitCode::Success as u8),
        Err(err) => {
            eprintln!("{err}");
            ProcessExitCode::from(bijux_atlas_core::ExitCode::Internal as u8)
        }
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let _output_mode = (cli.json, cli.quiet, cli.verbose, cli.trace);
    if cli.bijux_plugin_metadata {
        emit_plugin_metadata(cli.json)?;
        return Ok(());
    }

    let command = cli
        .command
        .ok_or_else(|| "missing command; see --help".to_string())?;

    match command {
        Commands::Catalog { command } => match command {
            CatalogCommand::Validate { path } => validate_catalog(path),
        },
        Commands::Dataset { command } => match command {
            DatasetCommand::Validate {
                root,
                release,
                species,
                assembly,
            } => validate_dataset(root, &release, &species, &assembly),
        },
        Commands::Ingest {
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
        } => run_ingest(IngestCliArgs {
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
        }),
        Commands::InspectDb { db, sample_rows } => inspect_db(db, sample_rows),
        Commands::ExplainQuery {
            db,
            gene_id,
            name,
            name_prefix,
            biotype,
            region,
            limit,
            allow_full_scan,
        } => explain_query(ExplainQueryArgs {
            db,
            gene_id,
            name,
            name_prefix,
            biotype,
            region,
            limit,
            allow_full_scan,
        }),
        Commands::Smoke {
            root,
            dataset,
            golden_queries,
            write_snapshot,
            snapshot_out,
        } => smoke_dataset(root, &dataset, golden_queries, write_snapshot, snapshot_out),
    }
}

fn emit_plugin_metadata(machine_json: bool) -> Result<(), String> {
    let payload = json!({
        "name": "bijux-atlas",
        "version": env!("CARGO_PKG_VERSION"),
        "compatible_umbrella": ">=0.1.0,<0.2.0",
        "build_hash": option_env!("BIJUX_BUILD_HASH").unwrap_or("dev"),
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

fn run_ingest(args: IngestCliArgs) -> Result<(), String> {
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
        seqid_policy: SeqidNormalizationPolicy::from_aliases(parse_alias_map(&args.seqid_aliases)),
    })
    .map_err(|e| e.to_string())?;

    println!("ingest manifest: {}", result.manifest_path.display());
    println!("ingest sqlite: {}", result.sqlite_path.display());
    println!(
        "ingest anomaly report: {}",
        result.anomaly_report_path.display()
    );
    Ok(())
}

fn inspect_db(db: PathBuf, sample_rows: usize) -> Result<(), String> {
    let conn = Connection::open(db).map_err(|e| e.to_string())?;
    let schema_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    println!("schema_version={schema_version}");

    let mut idx_stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|e| e.to_string())?;
    let indexes = idx_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    println!(
        "indexes={}",
        serde_json::to_string(&indexes).map_err(|e| e.to_string())?
    );

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM gene_summary", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    println!("gene_count={count}");

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
    println!(
        "sample_rows={}",
        serde_json::to_string(&rows).map_err(|e| e.to_string())?
    );
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

fn explain_query(args: ExplainQueryArgs) -> Result<(), String> {
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
    for l in lines {
        println!("{l}");
    }
    Ok(())
}

fn smoke_dataset(
    root: PathBuf,
    dataset: &str,
    golden_queries: PathBuf,
    write_snapshot: bool,
    snapshot_out: PathBuf,
) -> Result<(), String> {
    let (release, species, assembly) = parse_dataset_id(dataset)?;
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
        let req = query_request_from_json(body)?;
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

    println!("smoke: OK dataset={dataset} queries={}", out.len());
    Ok(())
}

fn parse_dataset_id(dataset: &str) -> Result<(String, String, String), String> {
    let parts: Vec<&str> = dataset.split('/').collect();
    if parts.len() != 3 {
        return Err("dataset must be release/species/assembly".to_string());
    }
    Ok((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

fn query_request_from_json(v: &Value) -> Result<GeneQueryRequest, String> {
    let gene_id = v
        .get("gene_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let name = v
        .get("name")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let name_prefix = v
        .get("name_prefix")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let biotype = v
        .get("biotype")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let region = if let Some(raw) = v.get("region").and_then(Value::as_str) {
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
    let limit = v.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
    let allow_full_scan = v
        .get("allow_full_scan")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    Ok(GeneQueryRequest {
        fields: GeneFields::default(),
        filter: GeneFilter {
            gene_id,
            name,
            name_prefix,
            biotype,
            region,
        },
        limit,
        cursor: None,
        allow_full_scan,
    })
}

fn parse_alias_map(input: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for pair in input.split(',') {
        let p = pair.trim();
        if p.is_empty() {
            continue;
        }
        if let Some((k, v)) = p.split_once('=') {
            out.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

fn validate_catalog(path: PathBuf) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let catalog: Catalog = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    catalog.validate_sorted().map_err(|e| e.to_string())?;
    println!("catalog validation: OK");
    Ok(())
}

fn validate_dataset(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::artifact_paths(&root, &dataset);

    let manifest_raw = fs::read_to_string(&paths.manifest).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    manifest.validate_strict().map_err(|e| e.to_string())?;

    check_sha(&paths.gff3, &manifest.checksums.gff3_sha256)?;
    check_sha(&paths.fasta, &manifest.checksums.fasta_sha256)?;
    check_sha(&paths.fai, &manifest.checksums.fai_sha256)?;
    check_sha(&paths.sqlite, &manifest.checksums.sqlite_sha256)?;

    let sqlite_bytes = fs::read(&paths.sqlite).map_err(|e| e.to_string())?;
    if !sqlite_bytes.starts_with(b"SQLite format 3\0") {
        return Err("sqlite artifact does not start with SQLite header".to_string());
    }

    if manifest.stats.gene_count == 0 {
        return Err("manifest gene_count must be > 0".to_string());
    }

    println!("dataset validation: OK");
    Ok(())
}

fn check_sha(path: &PathBuf, expected: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!(
            "checksum mismatch for {}: expected {}, got {}",
            path.display(),
            expected,
            actual
        ));
    }
    Ok(())
}
