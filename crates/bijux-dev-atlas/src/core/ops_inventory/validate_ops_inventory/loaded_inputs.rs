struct LoadedOpsInventoryValidationInputs {
    inventory: OpsInventory,
    k8s_install_matrix: K8sInstallMatrix,
    observe_alerts: ObserveCatalog,
    observe_slos: ObserveSloDefinitions,
    observe_drills: ObserveDrillCatalog,
    observe_readiness: ObserveReadiness,
    observe_telemetry_index: ObserveTelemetryIndex,
    datasets_manifest_lock: DatasetManifestLock,
    datasets_promotion_rules: DatasetPromotionRules,
    datasets_qc_metadata: DatasetQcMetadata,
    datasets_fixture_policy: DatasetFixturePolicy,
    datasets_rollback_policy: DatasetRollbackPolicy,
    datasets_index: DatasetIndex,
    datasets_lineage: DatasetLineage,
    e2e_suites: E2eSuitesManifest,
    e2e_scenarios: E2eScenariosManifest,
    e2e_expectations: E2eExpectations,
    e2e_fixture_allowlist: E2eFixtureAllowlist,
    e2e_reproducibility: E2eReproducibilityPolicy,
    e2e_taxonomy: E2eTaxonomy,
    e2e_summary: E2eSummary,
    e2e_coverage: E2eCoverageMatrix,
    report_evidence_levels: ReportEvidenceLevels,
    report_readiness: ReportReadinessScore,
    report_diff: ReportDiff,
    report_history: ReportHistoricalComparison,
    report_bundle: ReportReleaseEvidenceBundle,
    load_suites: LoadSuitesManifest,
    load_query_lock: LoadQueryLock,
    load_seed_policy: LoadSeedPolicy,
    load_query_catalog: LoadQueryPackCatalog,
    load_summary: LoadSummary,
    load_drift_report: LoadDriftReport,
    pins_manifest: PinsManifest,
    gates_manifest: GatesManifest,
}

fn load_ops_inventory_validation_inputs(
    repo_root: &Path,
    errors: &mut Vec<String>,
) -> Option<LoadedOpsInventoryValidationInputs> {
    let inventory = match load_ops_inventory(repo_root) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let k8s_install_matrix =
        match load_json::<K8sInstallMatrix>(repo_root, OPS_K8S_INSTALL_MATRIX_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let observe_alerts =
        match load_json::<ObserveCatalog>(repo_root, OPS_OBSERVE_ALERT_CATALOG_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let observe_slos =
        match load_json::<ObserveSloDefinitions>(repo_root, OPS_OBSERVE_SLO_DEFINITIONS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let observe_drills =
        match load_json::<ObserveDrillCatalog>(repo_root, OPS_OBSERVE_TELEMETRY_DRILLS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let observe_readiness =
        match load_json::<ObserveReadiness>(repo_root, OPS_OBSERVE_READINESS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let observe_telemetry_index =
        match load_json::<ObserveTelemetryIndex>(repo_root, OPS_OBSERVE_TELEMETRY_INDEX_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let datasets_manifest_lock =
        match load_json::<DatasetManifestLock>(repo_root, OPS_DATASETS_MANIFEST_LOCK_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let datasets_promotion_rules =
        match load_json::<DatasetPromotionRules>(repo_root, OPS_DATASETS_PROMOTION_RULES_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let datasets_qc_metadata =
        match load_json::<DatasetQcMetadata>(repo_root, OPS_DATASETS_QC_METADATA_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let datasets_fixture_policy =
        match load_json::<DatasetFixturePolicy>(repo_root, OPS_DATASETS_FIXTURE_POLICY_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let datasets_rollback_policy =
        match load_json::<DatasetRollbackPolicy>(repo_root, OPS_DATASETS_ROLLBACK_POLICY_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let datasets_index = match load_json::<DatasetIndex>(repo_root, OPS_DATASETS_INDEX_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let datasets_lineage = match load_json::<DatasetLineage>(repo_root, OPS_DATASETS_LINEAGE_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let e2e_suites = match load_json::<E2eSuitesManifest>(repo_root, OPS_E2E_SUITES_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let e2e_scenarios = match load_json::<E2eScenariosManifest>(repo_root, OPS_E2E_SCENARIOS_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let e2e_expectations = match load_json::<E2eExpectations>(repo_root, OPS_E2E_EXPECTATIONS_PATH)
    {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let e2e_fixture_allowlist =
        match load_json::<E2eFixtureAllowlist>(repo_root, OPS_E2E_FIXTURE_ALLOWLIST_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let e2e_reproducibility =
        match load_json::<E2eReproducibilityPolicy>(repo_root, OPS_E2E_REPRODUCIBILITY_POLICY_PATH)
        {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let e2e_taxonomy = match load_json::<E2eTaxonomy>(repo_root, OPS_E2E_TAXONOMY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let e2e_summary = match load_json::<E2eSummary>(repo_root, OPS_E2E_SUMMARY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let e2e_coverage = match load_json::<E2eCoverageMatrix>(repo_root, OPS_E2E_COVERAGE_MATRIX_PATH)
    {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let report_evidence_levels =
        match load_json::<ReportEvidenceLevels>(repo_root, OPS_REPORT_EVIDENCE_LEVELS_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let report_readiness =
        match load_json::<ReportReadinessScore>(repo_root, OPS_REPORT_READINESS_SCORE_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let report_diff = match load_json::<ReportDiff>(repo_root, OPS_REPORT_DIFF_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let report_history =
        match load_json::<ReportHistoricalComparison>(repo_root, OPS_REPORT_HISTORY_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let report_bundle =
        match load_json::<ReportReleaseEvidenceBundle>(repo_root, OPS_REPORT_RELEASE_BUNDLE_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let load_suites =
        match load_json::<LoadSuitesManifest>(repo_root, OPS_LOAD_SUITES_MANIFEST_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let load_query_lock = match load_json::<LoadQueryLock>(repo_root, OPS_LOAD_QUERY_LOCK_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let load_seed_policy = match load_json::<LoadSeedPolicy>(repo_root, OPS_LOAD_SEED_POLICY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let load_query_catalog =
        match load_json::<LoadQueryPackCatalog>(repo_root, OPS_LOAD_QUERY_PACK_CATALOG_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let load_summary = match load_json::<LoadSummary>(repo_root, OPS_LOAD_SUMMARY_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let load_drift_report =
        match load_json::<LoadDriftReport>(repo_root, OPS_LOAD_DRIFT_REPORT_PATH) {
            Ok(value) => value,
            Err(err) => {
                errors.push(err);
                return None;
            }
        };
    let pins_manifest = match load_pins_manifest(repo_root) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    let gates_manifest = match load_json::<GatesManifest>(repo_root, OPS_GATES_PATH) {
        Ok(value) => value,
        Err(err) => {
            errors.push(err);
            return None;
        }
    };
    Some(LoadedOpsInventoryValidationInputs {
        inventory,
        k8s_install_matrix,
        observe_alerts,
        observe_slos,
        observe_drills,
        observe_readiness,
        observe_telemetry_index,
        datasets_manifest_lock,
        datasets_promotion_rules,
        datasets_qc_metadata,
        datasets_fixture_policy,
        datasets_rollback_policy,
        datasets_index,
        datasets_lineage,
        e2e_suites,
        e2e_scenarios,
        e2e_expectations,
        e2e_fixture_allowlist,
        e2e_reproducibility,
        e2e_taxonomy,
        e2e_summary,
        e2e_coverage,
        report_evidence_levels,
        report_readiness,
        report_diff,
        report_history,
        report_bundle,
        load_suites,
        load_query_lock,
        load_seed_policy,
        load_query_catalog,
        load_summary,
        load_drift_report,
        pins_manifest,
        gates_manifest,
    })
}
