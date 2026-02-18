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

fn verify_ingest_inputs(
    gff3: PathBuf,
    fasta: PathBuf,
    fai: PathBuf,
    output_root: PathBuf,
    allow_network_inputs: bool,
    resume: bool,
    output_mode: OutputMode,
) -> Result<(), String> {
    let verified = resolve_verify_and_lock_inputs(
        &gff3,
        &fasta,
        &fai,
        &output_root,
        allow_network_inputs,
        resume,
    )?;
    command_output_adapters::emit_ok(
        output_mode,
        json!({
            "command": "atlas ingest verify-inputs",
            "status": "ok",
            "inputs_lockfile": verified.lockfile_path,
            "resolved": {
                "gff3": verified.gff3_path,
                "fasta": verified.fasta_path,
                "fai": verified.fai_path
            }
        }),
    )?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct InputLockfile {
    schema_version: u64,
    created_at_epoch_seconds: u64,
    sources: Vec<InputLockSource>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct InputLockSource {
    kind: String,
    original: String,
    resolved_url: String,
    output_path: String,
    checksum_sha256: String,
    expected_size_bytes: u64,
    original_filename: String,
    mirrors: Vec<String>,
}

#[derive(Debug)]
struct VerifiedInputPaths {
    gff3_path: PathBuf,
    fasta_path: PathBuf,
    fai_path: PathBuf,
    lockfile_path: PathBuf,
}

fn resolve_verify_and_lock_inputs(
    gff3: &std::path::Path,
    fasta: &std::path::Path,
    fai: &std::path::Path,
    output_root: &std::path::Path,
    allow_network_inputs: bool,
    resume: bool,
) -> Result<VerifiedInputPaths, String> {
    let ingest_inputs_dir = output_root.join("_ingest_inputs");
    let tmp_dir = ingest_inputs_dir.join(".tmp");
    let quarantine_dir = ingest_inputs_dir.join("quarantine");
    let lockfile_path = ingest_inputs_dir.join("inputs.lock.json");
    fs::create_dir_all(&ingest_inputs_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&quarantine_dir).map_err(|e| e.to_string())?;

    let specs = [("gff3", gff3), ("fasta", fasta), ("fai", fai)];
    if resume && lockfile_path.exists() {
        let existing: InputLockfile = serde_json::from_slice(
            &fs::read(&lockfile_path).map_err(|e| format!("read lockfile failed: {e}"))?,
        )
        .map_err(|e| format!("parse lockfile failed: {e}"))?;
        for (kind, src) in specs {
            let src_text = src.to_string_lossy().to_string();
            let entry = existing
                .sources
                .iter()
                .find(|x| x.kind == kind)
                .ok_or_else(|| format!("resume lockfile missing entry for {kind}"))?;
            if entry.original != src_text {
                return Err(format!(
                    "resume lockfile mismatch for {kind}: expected original `{}` got `{}`",
                    entry.original, src_text
                ));
            }
            let p = PathBuf::from(&entry.output_path);
            if !p.exists() {
                return Err(format!("resume file missing for {kind}: {}", p.display()));
            }
            let bytes = fs::read(&p).map_err(|e| e.to_string())?;
            let hash = sha256_hex(&bytes);
            if hash != entry.checksum_sha256 || bytes.len() as u64 != entry.expected_size_bytes {
                return Err(format!(
                    "resume lockfile TOCTOU mismatch for {kind}: hash/size changed"
                ));
            }
        }
        return Ok(VerifiedInputPaths {
            gff3_path: PathBuf::from(
                existing
                    .sources
                    .iter()
                    .find(|x| x.kind == "gff3")
                    .ok_or_else(|| "lockfile missing gff3".to_string())?
                    .output_path
                    .clone(),
            ),
            fasta_path: PathBuf::from(
                existing
                    .sources
                    .iter()
                    .find(|x| x.kind == "fasta")
                    .ok_or_else(|| "lockfile missing fasta".to_string())?
                    .output_path
                    .clone(),
            ),
            fai_path: PathBuf::from(
                existing
                    .sources
                    .iter()
                    .find(|x| x.kind == "fai")
                    .ok_or_else(|| "lockfile missing fai".to_string())?
                    .output_path
                    .clone(),
            ),
            lockfile_path,
        });
    }

    let mut lock_sources = Vec::new();
    let mut resolved_paths = std::collections::HashMap::new();
    for (kind, src) in specs {
        let src_text = src.to_string_lossy().to_string();
        let r = resolve_single_input(
            kind,
            &src_text,
            &ingest_inputs_dir,
            &tmp_dir,
            &quarantine_dir,
            allow_network_inputs,
        )?;
        resolved_paths.insert(kind.to_string(), r.0.clone());
        lock_sources.push(r.1);
    }
    let lock = InputLockfile {
        schema_version: 1,
        created_at_epoch_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs(),
        sources: lock_sources,
    };
    let lock_tmp = lockfile_path.with_extension("json.tmp");
    fs::write(
        &lock_tmp,
        canonical::stable_json_bytes(&lock).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(&lock_tmp, &lockfile_path).map_err(|e| e.to_string())?;
    Ok(VerifiedInputPaths {
        gff3_path: resolved_paths
            .remove("gff3")
            .ok_or_else(|| "missing resolved gff3".to_string())?,
        fasta_path: resolved_paths
            .remove("fasta")
            .ok_or_else(|| "missing resolved fasta".to_string())?,
        fai_path: resolved_paths
            .remove("fai")
            .ok_or_else(|| "missing resolved fai".to_string())?,
        lockfile_path,
    })
}

fn resolve_single_input(
    kind: &str,
    original: &str,
    ingest_inputs_dir: &std::path::Path,
    tmp_dir: &std::path::Path,
    quarantine_dir: &std::path::Path,
    allow_network_inputs: bool,
) -> Result<(PathBuf, InputLockSource), String> {
    let (resolved_url, source_bytes, original_filename) = if original.starts_with("http://")
        || original.starts_with("https://")
    {
        if !allow_network_inputs {
            return Err(format!(
                "network input forbidden by policy for {kind}; rerun with --allow-network-inputs"
            ));
        }
        let resp = reqwest::blocking::get(original).map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("download failed for {kind}: {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| e.to_string())?.to_vec();
        let filename = original.rsplit('/').next().unwrap_or(kind).to_string();
        (original.to_string(), bytes, filename)
    } else if original.starts_with("s3://") {
        if !allow_network_inputs {
            return Err(format!(
                "network input forbidden by policy for {kind}; rerun with --allow-network-inputs"
            ));
        }
        let endpoint = std::env::var("ATLAS_S3_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string());
        let key = original.trim_start_matches("s3://");
        let url = format!("{}/{}", endpoint.trim_end_matches('/'), key);
        let resp = reqwest::blocking::get(&url).map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("download failed for {kind}: {}", resp.status()));
        }
        let bytes = resp.bytes().map_err(|e| e.to_string())?.to_vec();
        let filename = key.rsplit('/').next().unwrap_or(kind).to_string();
        (url, bytes, filename)
    } else {
        let path = if let Some(p) = original.strip_prefix("file://") {
            PathBuf::from(p)
        } else {
            PathBuf::from(original)
        };
        let bytes = fs::read(&path).map_err(|e| format!("read local input failed: {e}"))?;
        let filename = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or(kind)
            .to_string();
        (path.display().to_string(), bytes, filename)
    };

    let (normalized_bytes, final_name) = match decompress_if_needed(original_filename, &source_bytes)
    {
        Ok(v) => v,
        Err(e) => {
            let q = quarantine_dir.join(format!(
                "{kind}-decode-{}.bad",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|x| x.to_string())?
                    .as_secs()
            ));
            fs::write(&q, &source_bytes).map_err(|x| x.to_string())?;
            return Err(format!(
                "decompression failed for {kind}: {e}; quarantined at {}",
                q.display()
            ));
        }
    };
    let final_path = ingest_inputs_dir.join(format!("{kind}-{final_name}"));
    let tmp_path = tmp_dir.join(format!("{kind}-{final_name}.part"));
    fs::write(&tmp_path, &normalized_bytes).map_err(|e| e.to_string())?;
    let verify_bytes = fs::read(&tmp_path).map_err(|e| e.to_string())?;
    if verify_bytes != normalized_bytes {
        let q = quarantine_dir.join(format!(
            "{kind}-{}.bad",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs()
        ));
        let _ = fs::rename(&tmp_path, &q);
        return Err(format!(
            "download verification failed for {kind}; quarantined at {}",
            q.display()
        ));
    }
    fs::rename(&tmp_path, &final_path).map_err(|e| e.to_string())?;
    let checksum = sha256_hex(&normalized_bytes);
    let source = InputLockSource {
        kind: kind.to_string(),
        original: original.to_string(),
        resolved_url,
        output_path: final_path.display().to_string(),
        checksum_sha256: checksum,
        expected_size_bytes: normalized_bytes.len() as u64,
        original_filename: final_name.clone(),
        mirrors: vec![],
    };
    Ok((final_path, source))
}

fn decompress_if_needed(filename: String, bytes: &[u8]) -> Result<(Vec<u8>, String), String> {
    if filename.ends_with(".gz") {
        let mut decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(bytes));
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut out).map_err(|e| e.to_string())?;
        return Ok((out, filename.trim_end_matches(".gz").to_string()));
    }
    if filename.ends_with(".zst") {
        let mut decoder =
            zstd::stream::read::Decoder::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut out).map_err(|e| e.to_string())?;
        return Ok((out, filename.trim_end_matches(".zst").to_string()));
    }
    Ok((bytes.to_vec(), filename))
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::resolve_verify_and_lock_inputs;
    use std::fs;

    #[test]
    fn resume_fails_on_lockfile_toc_tou_hash_mismatch() {
        let td = tempfile::tempdir().expect("tmp");
        let root = td.path().join("out");
        let src = td.path().join("genes.gff3");
        let fasta = td.path().join("genome.fa");
        let fai = td.path().join("genome.fa.fai");
        fs::write(&src, "chr1\ts\tgene\t1\t2\t.\t+\t.\tID=g1\n").expect("write gff3");
        fs::write(&fasta, ">chr1\nAC\n").expect("write fasta");
        fs::write(&fai, "chr1\t2\t0\t2\t3\n").expect("write fai");

        let first = resolve_verify_and_lock_inputs(&src, &fasta, &fai, &root, false, false)
            .expect("initial lock");
        fs::write(&first.gff3_path, "tampered").expect("tamper");
        let err =
            resolve_verify_and_lock_inputs(&src, &fasta, &fai, &root, false, true).expect_err(
                "resume must fail when file hash diverges from lockfile (TOCTOU protection)",
            );
        assert!(err.contains("TOCTOU mismatch"));
    }

    #[test]
    fn corrupted_gzip_input_is_quarantined() {
        let td = tempfile::tempdir().expect("tmp");
        let root = td.path().join("out");
        let src = td.path().join("genes.gff3.gz");
        let fasta = td.path().join("genome.fa");
        let fai = td.path().join("genome.fa.fai");
        fs::write(&src, "not-a-gzip").expect("write bad gzip");
        fs::write(&fasta, ">chr1\nAC\n").expect("write fasta");
        fs::write(&fai, "chr1\t2\t0\t2\t3\n").expect("write fai");

        let err = resolve_verify_and_lock_inputs(&src, &fasta, &fai, &root, false, false)
            .expect_err("bad gzip must fail");
        assert!(err.contains("quarantined"));
        let quarantine = root.join("_ingest_inputs").join("quarantine");
        let entries = fs::read_dir(quarantine).expect("read quarantine");
        assert!(entries.count() > 0);
    }

    #[test]
    fn network_inputs_are_forbidden_by_default() {
        let td = tempfile::tempdir().expect("tmp");
        let root = td.path().join("out");
        let err = resolve_verify_and_lock_inputs(
            std::path::Path::new("https://example.invalid/genes.gff3"),
            std::path::Path::new("https://example.invalid/genome.fa"),
            std::path::Path::new("https://example.invalid/genome.fa.fai"),
            &root,
            false,
            false,
        )
        .expect_err("network should be blocked unless explicitly allowed");
        assert!(err.contains("network input forbidden by policy"));
    }
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
