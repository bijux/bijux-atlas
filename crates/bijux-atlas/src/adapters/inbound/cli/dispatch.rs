// SPDX-License-Identifier: Apache-2.0

use super::actions::{
    explain_query, explain_query_from_query_text, export_query_rows, inspect_dataset, inspect_db,
    print_completion, print_config, print_version, run_ingest, run_query, smoke_dataset,
    ExplainQueryArgs,
};
use super::ingest_inputs::verify_ingest_inputs;
use super::operations;
use super::*;

pub(super) fn run_atlas_command(
    command: AtlasCommand,
    log_flags: LogFlags,
    output_mode: OutputMode,
) -> Result<(), CliError> {
    match command {
        AtlasCommand::Validate { root, release, species, assembly, deep } => {
            operations::validate_dataset(root, &release, &species, &assembly, deep, output_mode)
                .map_err(CliError::from_action_error)
        }
        AtlasCommand::Version => {
            print_version(log_flags.verbose > 0, output_mode).map_err(CliError::from_action_error)
        }
        AtlasCommand::Completion { shell } => {
            print_completion(shell);
            Ok(())
        }
        AtlasCommand::PrintConfig { canonical } => {
            print_config(canonical, output_mode).map_err(CliError::from_action_error)
        }
        AtlasCommand::Catalog { command } => match command {
            CatalogCommand::Validate { path } => {
                operations::validate_catalog(path, output_mode).map_err(CliError::from_action_error)
            }
            CatalogCommand::Publish { store_root, catalog, dry_run, explain } => {
                operations::publish_catalog(store_root, catalog, dry_run, explain, output_mode)
                    .map_err(CliError::from_action_error)
            }
            CatalogCommand::Rollback { store_root, release, species, assembly } => {
                operations::rollback_catalog(store_root, &release, &species, &assembly, output_mode)
                    .map_err(CliError::from_action_error)
            }
            CatalogCommand::Promote { store_root, release, species, assembly } => {
                operations::promote_catalog(store_root, &release, &species, &assembly, output_mode)
                    .map_err(CliError::from_action_error)
            }
            CatalogCommand::LatestAliasUpdate { store_root, release, species, assembly } => {
                operations::update_latest_alias(
                    store_root,
                    &release,
                    &species,
                    &assembly,
                    output_mode,
                )
                .map_err(CliError::from_action_error)
            }
        },
        AtlasCommand::Dataset { command } => match command {
            DatasetCommand::Verify { root, release, species, assembly, deep } => {
                operations::validate_dataset(root, &release, &species, &assembly, deep, output_mode)
                    .map_err(CliError::from_action_error)
            }
            DatasetCommand::Validate { root, release, species, assembly } => {
                operations::validate_dataset(
                    root,
                    &release,
                    &species,
                    &assembly,
                    false,
                    output_mode,
                )
                .map_err(CliError::from_action_error)
            }
            DatasetCommand::Publish {
                source_root,
                store_root,
                release,
                species,
                assembly,
                dry_run,
                explain,
            } => operations::publish_dataset(
                source_root,
                store_root,
                &release,
                &species,
                &assembly,
                dry_run,
                explain,
                output_mode,
            )
            .map_err(CliError::from_action_error),
            DatasetCommand::Pack { root, release, species, assembly, out } => {
                operations::pack_dataset(root, &release, &species, &assembly, out, output_mode)
                    .map_err(CliError::from_action_error)
            }
            DatasetCommand::VerifyPack { pack } => {
                operations::verify_pack(pack, output_mode).map_err(CliError::from_action_error)
            }
        },
        AtlasCommand::Diff { command } => match command {
            DiffCommand::Build {
                root,
                from_release,
                to_release,
                species,
                assembly,
                out_dir,
                max_inline_items,
            } => operations::build_release_diff(
                operations::BuildReleaseDiffArgs {
                    root,
                    from_release,
                    to_release,
                    species,
                    assembly,
                    out_dir,
                    max_inline_items,
                },
                output_mode,
            )
            .map_err(CliError::from_action_error),
        },
        AtlasCommand::Gc { command } => match command {
            GcCommand::Plan { store_root, catalog, pins } => {
                operations::gc_plan(store_root, catalog, pins, output_mode)
                    .map_err(CliError::from_action_error)
            }
            GcCommand::Apply { store_root, catalog, pins, confirm } => {
                operations::gc_apply(store_root, catalog, pins, confirm, output_mode)
                    .map_err(CliError::from_action_error)
            }
        },
        AtlasCommand::Policy { command } => match command {
            PolicyCommand::Validate => {
                operations::validate_policy(output_mode).map_err(CliError::from_action_error)
            }
            PolicyCommand::Explain { mode } => operations::explain_policy(
                mode.map(|m| match m {
                    PolicyModeCli::Strict => crate::domain::policy::PolicyMode::Strict,
                    PolicyModeCli::Compat => crate::domain::policy::PolicyMode::Compat,
                    PolicyModeCli::Dev => crate::domain::policy::PolicyMode::Dev,
                }),
                output_mode,
            )
            .map_err(CliError::from_action_error),
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
            allow_network_inputs,
            resume,
            fasta_scanning,
            fasta_scan_max_bases,
            emit_shards,
            shard_partitions,
            sharding_plan,
            emit_normalized_debug,
            normalized_replay,
            prod_mode,
            dry_run,
            explain,
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
                allow_network_inputs,
                resume,
                fasta_scanning,
                fasta_scan_max_bases,
                emit_shards,
                shard_partitions,
                sharding_plan,
                emit_normalized_debug,
                normalized_replay,
                prod_mode,
                dry_run,
                explain,
            },
            output_mode,
        )
        .map_err(CliError::from_action_error),
        AtlasCommand::IngestVerifyInputs {
            gff3,
            fasta,
            fai,
            output_root,
            allow_network_inputs,
            resume,
        } => verify_ingest_inputs(
            gff3,
            fasta,
            fai,
            output_root,
            allow_network_inputs,
            resume,
            output_mode,
        )
        .map_err(CliError::from_action_error),
        AtlasCommand::IngestReplay { normalized } => {
            let counts = replay_normalized_counts(&normalized)
                .map_err(|e| CliError::internal(e.to_string()))?;
            output::emit_ok(
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
            .map_err(CliError::from_action_error)
        }
        AtlasCommand::IngestNormalizedDiff { base, target } => {
            let (removed, added) = diff_normalized_ids(&base, &target)
                .map_err(|e| CliError::internal(e.to_string()))?;
            output::emit_ok(
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
            .map_err(CliError::from_action_error)
        }
        AtlasCommand::IngestValidate { qc_report, thresholds } => {
            operations::validate_ingest_qc(qc_report, thresholds, output_mode)
                .map_err(CliError::from_action_error)
        }
        AtlasCommand::InspectDb { db, sample_rows } => {
            inspect_db(db, sample_rows, output_mode).map_err(CliError::from_action_error)
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
        .map_err(CliError::from_action_error),
        AtlasCommand::Explain { db, query, limit, allow_full_scan } => {
            explain_query_from_query_text(db, &query, limit, allow_full_scan, output_mode)
                .map_err(CliError::from_action_error)
        }
        AtlasCommand::Smoke { root, dataset, golden_queries, write_snapshot, snapshot_out } => {
            smoke_dataset(root, &dataset, golden_queries, write_snapshot, snapshot_out, output_mode)
                .map_err(CliError::from_action_error)
        }
        AtlasCommand::Openapi { command } => match command {
            OpenapiCommand::Generate { out } => output::run_openapi_generate(out, output_mode),
        }
        .map_err(CliError::from_action_error),
        AtlasCommand::Query { command } => match command {
            QueryCommand::Run {
                db,
                gene_id,
                name,
                name_prefix,
                biotype,
                region,
                limit,
                allow_full_scan,
            } => run_query(
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
            ),
            QueryCommand::Explain {
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
            ),
        }
        .map_err(CliError::from_action_error),
        AtlasCommand::Inspect { command } => match command {
            InspectCommand::Dataset { root, release, species, assembly } => {
                inspect_dataset(root, &release, &species, &assembly, output_mode)
            }
            InspectCommand::Db { db, sample_rows } => inspect_db(db, sample_rows, output_mode),
        }
        .map_err(CliError::from_action_error),
        AtlasCommand::Export { command } => match command {
            ExportCommand::Openapi { out } => output::run_openapi_generate(out, output_mode),
            ExportCommand::Query {
                db,
                out,
                format,
                gene_id,
                name,
                name_prefix,
                biotype,
                region,
                limit,
                allow_full_scan,
            } => export_query_rows(
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
                out,
                format,
                output_mode,
            ),
        }
        .map_err(CliError::from_action_error),
    }
}
