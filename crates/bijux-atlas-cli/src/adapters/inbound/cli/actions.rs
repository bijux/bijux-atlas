// SPDX-License-Identifier: Apache-2.0

use super::ingest_inputs::resolve_verify_and_lock_inputs;
use super::operations;
use super::*;
use bijux_atlas_query::{
    DuplicateTranscriptIdPolicy, FeatureIdUniquenessPolicy, TranscriptIdPolicy,
    UnknownFeaturePolicy,
};

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
