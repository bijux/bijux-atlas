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

fn run_bench_command(
    suite: &str,
    enforce_baseline: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    if enforce_baseline {
        cmd.env("ATLAS_QUERY_BENCH_ENFORCE", "1");
    }
    match suite {
        "query-patterns" => {
            cmd.args(["bench", "-p", "bijux-atlas-query", "--bench", "query_patterns"]);
        }
        "server-cache" => {
            cmd.args(["bench", "-p", "bijux-atlas-server", "--bench", "cache_manager"]);
        }
        "server-sequence" => {
            cmd.args(["bench", "-p", "bijux-atlas-server", "--bench", "sequence_fetch"]);
        }
        "server-diff" => {
            cmd.args(["bench", "-p", "bijux-atlas-server", "--bench", "diff_merge"]);
        }
        "server-bulkhead" => {
            cmd.args(["bench", "-p", "bijux-atlas-server", "--bench", "bulkhead_tuning"]);
        }
        other => {
            return Err(format!(
                "unknown bench suite `{other}`; supported: query-patterns, server-cache, server-sequence, server-diff, server-bulkhead"
            ));
        }
    }

    let status = cmd.status().map_err(|e| format!("failed to run cargo bench: {e}"))?;
    if !status.success() {
        return Err(format!("benchmark command failed with status {status}"));
    }
    command_output_adapters::emit_ok(
        output_mode,
        json!({
            "command":"atlas bench",
            "suite": suite,
            "status":"ok",
            "enforce_baseline": enforce_baseline
        }),
    )?;
    Ok(())
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
        command_output_adapters::emit_ok(output_mode, json!({"command":"atlas serve","status":"ok"}))?;
        Ok(())
    } else {
        Err(format!("atlas-server exited with status {status}"))
    }
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
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
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

fn doctor(output_mode: OutputMode) -> Result<(), String> {
    let mut checks = Vec::<Value>::new();

    let cache_dir = resolve_bijux_cache_dir();
    let cache_path = std::path::PathBuf::from(&cache_dir);
    let cache_exists = cache_path.exists();
    let cache_writable = if cache_exists {
        fs::metadata(&cache_path)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false)
    } else {
        false
    };
    checks.push(json!({
        "check":"cache_dir",
        "path": cache_dir,
        "ok": cache_exists && cache_writable
    }));

    let workspace_config = resolve_bijux_config_path(ConfigPathScope::Workspace);
    let user_config = resolve_bijux_config_path(ConfigPathScope::User);
    checks.push(json!({
        "check":"config_paths",
        "workspace_config": workspace_config,
        "user_config": user_config,
        "ok": true
    }));

    let store_root = std::env::var("ATLAS_STORE_ROOT").ok();
    let store_ok = store_root
        .as_ref()
        .map(|p| std::path::Path::new(p).exists())
        .unwrap_or(true);
    checks.push(json!({
        "check":"store_access",
        "atlas_store_root": store_root,
        "ok": store_ok
    }));

    let all_ok = checks
        .iter()
        .all(|c| c.get("ok").and_then(Value::as_bool).unwrap_or(false));
    command_output_adapters::emit_ok(
        output_mode,
        json!({"command":"atlas doctor","status": if all_ok {"ok"} else {"degraded"}, "checks": checks}),
    )?;
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

    let report_only = args.report_only || matches!(strictness, StrictnessMode::ReportOnly);
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
        report_only,
        fail_on_warn: args.strict,
        allow_overlap_gene_ids_across_contigs: args.allow_overlap_gene_ids_across_contigs,
        emit_shards: args.emit_shards,
        shard_partitions: args.shard_partitions,
        compute_gene_signatures: true,
        compute_contig_fractions: false,
        compute_transcript_spliced_length: false,
        compute_transcript_cds_length: false,
        dev_allow_auto_generate_fai: false,
        duplicate_transcript_id_policy: bijux_atlas_model::DuplicateTranscriptIdPolicy::Reject,
        transcript_id_policy: bijux_atlas_model::TranscriptIdPolicy::default(),
        unknown_feature_policy: bijux_atlas_model::UnknownFeaturePolicy::IgnoreWithWarning,
        feature_id_uniqueness_policy: bijux_atlas_model::FeatureIdUniquenessPolicy::Reject,
        reject_normalized_seqid_collisions: true,
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
