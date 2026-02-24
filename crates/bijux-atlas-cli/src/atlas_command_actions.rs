#[path = "atlas_command_actions/ingest_inputs.rs"]
mod ingest_inputs;

use ingest_inputs::{resolve_verify_and_lock_inputs, verify_ingest_inputs};

fn parse_query_text(query_text: &str) -> HashMap<String, String> {
    query_text
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k.is_empty() {
                return None;
            }
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

fn explain_query_from_query_text(
    db: PathBuf,
    query_text: &str,
    limit: usize,
    allow_full_scan: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    let parsed = parse_query_text(query_text);
    explain_query(
        ExplainQueryArgs {
            db,
            gene_id: parsed.get("gene_id").cloned(),
            name: parsed.get("name").cloned(),
            name_prefix: parsed.get("name_prefix").cloned(),
            biotype: parsed.get("biotype").cloned(),
            region: parsed.get("region").cloned(),
            limit,
            allow_full_scan,
        },
        output_mode,
    )
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
        let bytes = canonical::stable_json_bytes(&payload).map_err(|e| e.to_string())?;
        let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        println!("{text}");
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
        let bytes = canonical::stable_json_bytes(&payload).map_err(|e| e.to_string())?;
        let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        println!("{text}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

fn print_version(verbose: bool, output_mode: OutputMode) -> Result<(), String> {
    let payload = if verbose {
        json!({
            "plugin": {
                "name": "bijux-atlas",
                "version": env!("CARGO_PKG_VERSION"),
                "build_hash": option_env!("BIJUX_BUILD_HASH").unwrap_or("dev"),
                "rustc": option_env!("RUSTC_VERSION").unwrap_or("unknown")
            },
            "schemas": {
                "plugin_metadata_schema_version": "v1",
                "openapi_version": "v1"
            }
        })
    } else {
        json!({"name":"bijux-atlas","version": env!("CARGO_PKG_VERSION")})
    };
    command_output_adapters::emit_ok(output_mode, payload)?;
    Ok(())
}

fn plugin_metadata_payload() -> Value {
    json!({
        "schema_version": "v1",
        "name": "bijux-atlas",
        "version": env!("CARGO_PKG_VERSION"),
        "compatible_umbrella_min": UMBRELLA_MIN_VERSION,
        "compatible_umbrella_max_exclusive": UMBRELLA_MAX_EXCLUSIVE_VERSION,
        "compatible_umbrella": ">=0.1.0,<0.2.0",
        "build_hash": option_env!("BIJUX_BUILD_HASH").unwrap_or("dev"),
    })
}

fn enforce_umbrella_compatibility(version: &str) -> Result<(), CliError> {
    if !version_in_supported_range(version) {
        return Err(CliError {
            exit_code: bijux_atlas_core::ExitCode::Usage,
            machine: MachineError::new(
                "umbrella_incompatible",
                "umbrella version is outside plugin compatibility range",
            )
            .with_detail("version", version)
            .with_detail("min", UMBRELLA_MIN_VERSION)
            .with_detail("max_exclusive", UMBRELLA_MAX_EXCLUSIVE_VERSION),
        });
    }
    Ok(())
}

fn version_in_supported_range(version: &str) -> bool {
    let parts: Vec<_> = version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    matches!((parts[0], parts[1]), ("0", "1"))
}

fn print_config(canonical_out: bool, output_mode: OutputMode) -> Result<(), String> {
    let payload = json!({
        "workspace_config": resolve_bijux_config_path(ConfigPathScope::Workspace),
        "user_config": resolve_bijux_config_path(ConfigPathScope::User),
        "cache_dir": resolve_bijux_cache_dir(),
        "env": {
            "BIJUX_LOG_LEVEL": std::env::var("BIJUX_LOG_LEVEL").ok(),
            "BIJUX_CACHE_DIR": std::env::var("BIJUX_CACHE_DIR").ok(),
            "ATLAS_STORE_ROOT": std::env::var("ATLAS_STORE_ROOT").ok(),
        }
    });
    if output_mode.json {
        let text = if canonical_out {
            String::from_utf8(canonical::stable_json_bytes(&payload).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?
        } else {
            let bytes = canonical::stable_json_bytes(&payload).map_err(|e| e.to_string())?;
            String::from_utf8(bytes).map_err(|e| e.to_string())?
        };
        println!("{text}");
        return Ok(());
    }
    let text = if canonical_out {
        String::from_utf8(canonical::stable_json_bytes(&payload).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
    };
    println!("{text}");
    Ok(())
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
    if args.no_fai_check {
        return Err(
            "policy gate: --no-fai-check is forbidden in production; use --dev-auto-generate-fai for local development"
                .to_string(),
        );
    }
    let dataset =
        DatasetId::new(&args.release, &args.species, &args.assembly).map_err(|e| e.to_string())?;

    let strictness = match args.strictness {
        StrictnessCli::Strict => StrictnessMode::Strict,
        StrictnessCli::Compat => StrictnessMode::Lenient,
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

    let report_only = args.report_only || matches!(strictness, StrictnessMode::ReportOnly);
    let verified_inputs = resolve_verify_and_lock_inputs(
        &args.gff3,
        &args.fasta,
        &args.fai,
        &args.output_root,
        args.allow_network_inputs,
        args.resume,
    )?;
    let (policy_sharding_default, policy_max_shards) = read_sharding_policy_defaults();
    let sharding_plan = match args.sharding_plan.unwrap_or(policy_sharding_default) {
        ShardingPlanCli::None => ShardingPlan::None,
        ShardingPlanCli::Contig => ShardingPlan::Contig,
        ShardingPlanCli::RegionGrid => ShardingPlan::RegionGrid,
    };
    let result = ingest_dataset(&IngestOptions {
        gff3_path: verified_inputs.gff3_path,
        fasta_path: verified_inputs.fasta_path,
        fai_path: verified_inputs.fai_path,
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
        report_only,
        fail_on_warn: args.strict,
        allow_overlap_gene_ids_across_contigs: args.allow_overlap_gene_ids_across_contigs,
        dev_allow_auto_generate_fai: args.dev_auto_generate_fai,
        fasta_scanning_enabled: args.fasta_scanning,
        fasta_scan_max_bases: args.fasta_scan_max_bases,
        emit_shards: args.emit_shards,
        shard_partitions: args.shard_partitions,
        sharding_plan,
        max_shards: policy_max_shards,
        emit_normalized_debug: args.emit_normalized_debug,
        normalized_replay_mode: args.normalized_replay,
        prod_mode: args.prod_mode,
        compute_gene_signatures: true,
        compute_contig_fractions: false,
        compute_transcript_spliced_length: false,
        compute_transcript_cds_length: false,
        duplicate_transcript_id_policy: bijux_atlas_model::DuplicateTranscriptIdPolicy::Reject,
        transcript_id_policy: bijux_atlas_model::TranscriptIdPolicy::default(),
        unknown_feature_policy: bijux_atlas_model::UnknownFeaturePolicy::IgnoreWithWarning,
        feature_id_uniqueness_policy: bijux_atlas_model::FeatureIdUniquenessPolicy::Reject,
        reject_normalized_seqid_collisions: true,
        timestamp_policy: TimestampPolicy::DeterministicZero,
    })
    .map_err(|e| e.to_string())?;

    command_output_adapters::emit_ok(
        output_mode,
        json!({
            "command":"atlas ingest",
            "status":"ok",
            "report_only": report_only,
            "manifest": result.manifest_path,
            "sqlite": result.sqlite_path,
            "anomaly_report": result.anomaly_report_path
        }),
    )?;
    Ok(())
}

fn read_sharding_policy_defaults() -> (ShardingPlanCli, usize) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("configs/ops/sharding-policy.json");
    let raw = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => return (ShardingPlanCli::None, 512),
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return (ShardingPlanCli::None, 512),
    };
    let plan = match v
        .get("default_plan")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none")
    {
        "contig" => ShardingPlanCli::Contig,
        "region_grid" => ShardingPlanCli::RegionGrid,
        _ => ShardingPlanCli::None,
    };
    let max_shards = v
        .get("max_shards")
        .and_then(serde_json::Value::as_u64)
        .map(|x| x as usize)
        .unwrap_or(512);
    (plan, max_shards)
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
    command_output_adapters::emit_ok(
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
        dataset_key: None,
        allow_full_scan: args.allow_full_scan,
    };
    let query_class = classify_query(&req);
    let cost_units = bijux_atlas_query::estimate_work_units(&req);
    let lines = explain_query_plan(&conn, &req, &QueryLimits::default(), b"atlas-cli")
        .map_err(|e| e.to_string())?;
    command_output_adapters::emit_ok(
        output_mode,
        json!({
            "command":"atlas explain",
            "query_class": format!("{query_class:?}"),
            "estimated_cost_units": cost_units,
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
    let (release, species, assembly) = command_output_adapters::parse_dataset_id(dataset)?;
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
        let req = command_output_adapters::query_request_from_json(body)?;
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

    command_output_adapters::emit_ok(
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
