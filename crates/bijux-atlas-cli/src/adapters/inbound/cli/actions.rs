// SPDX-License-Identifier: Apache-2.0

use super::ingest_inputs::resolve_verify_and_lock_inputs;
use super::operations;
use super::*;
use bijux_atlas_model::dataset::ArtifactManifest;
use bijux_atlas_query::{
    query_genes, DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy, QueryLimits,
    TranscriptIdPolicy, UnknownFeaturePolicy,
};

use std::path::PathBuf;

pub(super) fn run_ingest(args: IngestCliArgs, output_mode: OutputMode) -> Result<(), String> {
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
    let ingest_options = IngestOptions {
        gff3_path: verified_inputs.gff3_path,
        fasta_path: verified_inputs.fasta_path,
        fai_path: verified_inputs.fai_path,
        output_root: args.output_root,
        build_hash: String::new(),
        dataset,
        strictness,
        duplicate_gene_id_policy,
        gene_identifier_policy,
        gene_name_policy: GeneNamePolicy::default(),
        biotype_policy: BiotypePolicy::default(),
        transcript_type_policy: TranscriptTypePolicy::default(),
        seqid_policy: SeqidNormalizationPolicy::from_aliases(operations::parse_alias_map(
            &args.seqid_aliases,
        )),
        max_threads: args.max_threads,
        report_only,
        fail_on_warn: args.strict,
        max_warn_anomalies: None,
        max_error_anomalies: None,
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
        duplicate_transcript_id_policy: DuplicateTranscriptIdPolicy::Reject,
        transcript_id_policy: TranscriptIdPolicy::default(),
        unknown_feature_policy: UnknownFeaturePolicy::IgnoreWithWarning,
        feature_id_uniqueness_policy: FeatureIdUniquenessPolicy::Reject,
        reject_normalized_seqid_collisions: true,
        timestamp_policy: TimestampPolicy::DeterministicZero,
    };

    if args.dry_run || args.explain {
        output::emit_ok(
            output_mode,
            json!({
                "command":"atlas ingest",
                "mode": if args.explain { "explain" } else { "dry-run" },
                "status":"ok",
                "dataset": ingest_options.dataset.canonical_string(),
                "report_only": ingest_options.report_only,
                "strictness": format!("{:?}", ingest_options.strictness),
                "sharding_plan": format!("{:?}", ingest_options.sharding_plan),
                "inputs": {
                    "gff3": ingest_options.gff3_path,
                    "fasta": ingest_options.fasta_path,
                    "fai": ingest_options.fai_path
                },
                "output_root": ingest_options.output_root,
                "writes_artifacts": false
            }),
        )?;
        return Ok(());
    }

    let result = ingest_dataset(&ingest_options).map_err(|e| e.to_string())?;

    output::emit_ok(
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
        .join("configs/sources/operations/ops/sharding-policy.json");
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

pub(super) fn inspect_db(
    db: PathBuf,
    sample_rows: usize,
    output_mode: OutputMode,
) -> Result<(), String> {
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
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas inspect db",
            "schema_version": schema_version,
            "indexes": indexes,
            "gene_count": count,
            "sample_rows": rows
        }),
    )?;
    Ok(())
}

pub(super) fn inspect_dataset(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::dataset::artifact_paths(&root, &dataset);
    let manifest_raw = fs::read_to_string(&paths.manifest).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas inspect dataset",
            "dataset": dataset.canonical_string(),
            "paths": {
                "manifest": paths.manifest,
                "sqlite": paths.sqlite,
                "derived_dir": paths.derived_dir,
            },
            "stats": manifest.stats,
            "identity": manifest.identity,
            "canonical_feature_counts": manifest.canonical_feature_counts,
        }),
    )?;
    Ok(())
}

pub(super) fn inspect_provenance(
    root: PathBuf,
    release: &str,
    species: &str,
    assembly: &str,
    output_mode: OutputMode,
) -> Result<(), String> {
    let dataset = DatasetId::new(release, species, assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::dataset::artifact_paths(&root, &dataset);
    let manifest_raw = fs::read_to_string(&paths.manifest).map_err(|e| e.to_string())?;
    let manifest: ArtifactManifest =
        serde_json::from_str(&manifest_raw).map_err(|e| e.to_string())?;
    let source_facts: Value =
        serde_json::from_str(&fs::read_to_string(&paths.source_facts).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let build_metadata: Value = serde_json::from_str(
        &fs::read_to_string(&paths.build_metadata).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let artifact_inventory: Value = serde_json::from_str(
        &fs::read_to_string(&paths.artifact_inventory).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let scientific_profile: Value = serde_json::from_str(
        &fs::read_to_string(&paths.scientific_profile).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    output::emit_ok(
        output_mode,
        json!({
            "command":"atlas inspect provenance",
            "dataset": dataset.canonical_string(),
            "build_evidence": {
                "manifest_identity": manifest.identity,
                "input_hashes": manifest.input_hashes,
                "normalized_input_identity_sha256": manifest.normalized_input_identity_sha256,
                "build_policy_version": manifest.build_policy_version,
                "reference_build_identity_sha256": manifest.reference_build_identity_sha256,
                "contig_naming_style": manifest.contig_naming_style,
                "scientific_prerequisites_status": manifest.scientific_prerequisites_status
            },
            "runtime_api_evidence": {
                "carrier": "http provenance envelope",
                "fields": ["dataset_hash", "release", "species", "assembly", "manifest_version", "db_schema_version", "dataset_signature_sha256"]
            },
            "source_facts": source_facts,
            "build_metadata": build_metadata,
            "artifact_inventory": artifact_inventory,
            "scientific_profile": scientific_profile,
        }),
    )?;
    Ok(())
}

pub(super) fn smoke_dataset(
    root: PathBuf,
    dataset: &str,
    golden_queries: PathBuf,
    write_snapshot: bool,
    snapshot_out: PathBuf,
    output_mode: OutputMode,
) -> Result<(), String> {
    let (release, species, assembly) = output::parse_dataset_id(dataset)?;
    let id = DatasetId::new(&release, &species, &assembly).map_err(|e| e.to_string())?;
    let paths = bijux_atlas_model::dataset::artifact_paths(&root, &id);
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
        let req = output::query_request_from_json(body)?;
        let resp = query_genes(&conn, &req, &QueryLimits::default(), b"smoke")
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

    output::emit_ok(
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
