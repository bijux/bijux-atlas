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
            CatalogCommand::Promote {
                store_root,
                release,
                species,
                assembly,
            } => artifact_validation::promote_catalog(
                store_root,
                &release,
                &species,
                &assembly,
                output_mode,
            )
            .map_err(CliError::internal),
            CatalogCommand::LatestAliasUpdate {
                store_root,
                release,
                species,
                assembly,
            } => artifact_validation::update_latest_alias(
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
        AtlasCommand::Diff { command } => match command {
            DiffCommand::Build {
                root,
                from_release,
                to_release,
                species,
                assembly,
                out_dir,
                max_inline_items,
            } => artifact_validation::build_release_diff(
                artifact_validation::BuildReleaseDiffArgs {
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
            .map_err(CliError::internal),
        },
        AtlasCommand::Gc { command } => match command {
            GcCommand::Plan {
                store_root,
                catalog,
                pins,
            } => artifact_validation::gc_plan(store_root, catalog, pins, output_mode)
                .map_err(CliError::internal),
            GcCommand::Apply {
                store_root,
                catalog,
                pins,
                confirm,
            } => artifact_validation::gc_apply(store_root, catalog, pins, confirm, output_mode)
                .map_err(CliError::internal),
        },
        AtlasCommand::Policy { command } => match command {
            PolicyCommand::Validate => {
                artifact_validation::validate_policy(output_mode).map_err(CliError::internal)
            }
            PolicyCommand::Explain { mode } => artifact_validation::explain_policy(
                mode.map(|m| match m {
                    PolicyModeCli::Strict => bijux_atlas_policies::PolicyMode::Strict,
                    PolicyModeCli::Compat => bijux_atlas_policies::PolicyMode::Compat,
                    PolicyModeCli::Dev => bijux_atlas_policies::PolicyMode::Dev,
                }),
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
            },
            output_mode,
        )
        .map_err(CliError::internal),
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

