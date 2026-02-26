pub(super) fn check_ops_evidence_bundle_discipline(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    let generated_lifecycle_rel = Path::new("ops/GENERATED_LIFECYCLE.md");
    let evidence_checklist_rel = Path::new("ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md");
    let mirror_policy_rel = Path::new("ops/_generated.example/MIRROR_POLICY.md");
    let allowlist_rel = Path::new("ops/_generated.example/ALLOWLIST.json");
    let ops_index_rel = Path::new("ops/_generated.example/ops-index.json");
    let scorecard_rel = Path::new("ops/_generated.example/scorecard.json");
    let bundle_rel = Path::new("ops/_generated.example/ops-evidence-bundle.json");
    let contract_audit_rel = Path::new("ops/_generated.example/contract-audit-report.json");
    let contract_graph_rel = Path::new("ops/_generated.example/contract-dependency-graph.json");
    let control_graph_diff_rel =
        Path::new("ops/_generated.example/control-graph-diff-report.json");
    let schema_drift_rel = Path::new("ops/_generated.example/schema-drift-report.json");
    let evidence_gap_rel = Path::new("ops/_generated.example/evidence-gap-report.json");
    let readiness_score_rel = Path::new("ops/report/generated/readiness-score.json");
    let historical_comparison_rel = Path::new("ops/report/generated/historical-comparison.json");
    let release_bundle_rel = Path::new("ops/report/generated/release-evidence-bundle.json");
    let gates_rel = Path::new("ops/inventory/gates.json");

    for rel in [
        generated_lifecycle_rel,
        evidence_checklist_rel,
        mirror_policy_rel,
        allowlist_rel,
        ops_index_rel,
        scorecard_rel,
        bundle_rel,
        contract_audit_rel,
        contract_graph_rel,
        control_graph_diff_rel,
        schema_drift_rel,
        evidence_gap_rel,
    ] {
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_EVIDENCE_REQUIRED_ARTIFACT_MISSING",
                format!("missing required evidence artifact `{}`", rel.display()),
                "generate and commit required evidence artifact",
                Some(rel),
            ));
        }
    }

    let mirror_policy_text = fs::read_to_string(ctx.repo_root.join(mirror_policy_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let generated_lifecycle_text = fs::read_to_string(ctx.repo_root.join(generated_lifecycle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for required in [
        "## Lifecycle Classes",
        "transient_runtime",
        "domain_derived",
        "curated_evidence",
        "## Retention Policy",
        "## Regeneration Triggers",
        "## Deterministic Ordering",
        "## Generator Contract Versioning",
    ] {
        if !generated_lifecycle_text.contains(required) {
            violations.push(violation(
                "OPS_GENERATED_LIFECYCLE_CONTRACT_INCOMPLETE",
                format!(
                    "generated lifecycle contract must include required section or marker `{required}`"
                ),
                "update ops/GENERATED_LIFECYCLE.md with complete lifecycle, retention, trigger, and versioning policy",
                Some(generated_lifecycle_rel),
            ));
        }
    }
    for required in [
        "## Required Artifacts",
        "## Completeness Rules",
        "## Lineage Rules",
        "## Blocking Conditions",
        "ops/_generated.example/evidence-gap-report.json",
        "ops/report/generated/release-evidence-bundle.json",
        "ops/report/generated/historical-comparison.json",
        "ops/report/generated/readiness-score.json",
    ] {
        if !fs::read_to_string(ctx.repo_root.join(evidence_checklist_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?
            .contains(required)
        {
            violations.push(violation(
                "OPS_EVIDENCE_COMPLETENESS_CHECKLIST_INCOMPLETE",
                format!("evidence completeness checklist must include `{required}`"),
                "update ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md with required evidence rules and artifacts",
                Some(evidence_checklist_rel),
            ));
        }
    }
    for required in [
        "ops-index.json",
        "ops-evidence-bundle.json",
        "scorecard.json",
        "ALLOWLIST.json",
        "contract-audit-report.json",
        "contract-dependency-graph.json",
        "inventory-index.json",
        "control-plane.snapshot.md",
        "control-graph-diff-report.json",
        "docs-drift-report.json",
        "schema-drift-report.json",
        "stack-drift-report.json",
        "ops/GENERATED_LIFECYCLE.md",
        "evidence-gap-report.json",
    ] {
        if !mirror_policy_text.contains(required) {
            violations.push(violation(
                "OPS_EVIDENCE_MIRROR_POLICY_INCOMPLETE",
                format!("mirror policy must declare mirrored artifact `{required}`"),
                "update MIRROR_POLICY.md mirrored artifact list",
                Some(mirror_policy_rel),
            ));
        }
    }

    let allowlist_text = fs::read_to_string(ctx.repo_root.join(allowlist_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlist_json: serde_json::Value =
        serde_json::from_str(&allowlist_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let allowlisted_files = allowlist_json
        .get("allowed_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if allowlisted_files.is_empty() {
        violations.push(violation(
            "OPS_EVIDENCE_ALLOWLIST_EMPTY",
            "ops/_generated.example/ALLOWLIST.json must declare non-empty `allowed_files`"
                .to_string(),
            "populate ALLOWLIST.json with exact committed files allowed under ops/_generated.example",
            Some(allowlist_rel),
        ));
    }
    let generated_example_root = ctx.repo_root.join("ops/_generated.example");
    if generated_example_root.exists() {
        let mut seen_files = BTreeSet::new();
        for file in walk_files(&generated_example_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            seen_files.insert(rel_str.clone());
            if !allowlisted_files.contains(&rel_str) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_MISSING_FILE",
                    format!(
                        "committed file `{}` is not declared in ops/_generated.example/ALLOWLIST.json",
                        rel.display()
                    ),
                    "update ALLOWLIST.json when adding or removing curated evidence artifacts",
                    Some(allowlist_rel),
                ));
            }
            if is_binary_like_file(&file)? {
                violations.push(violation(
                    "OPS_EVIDENCE_BINARY_FORBIDDEN",
                    format!(
                        "binary file is forbidden under ops/_generated.example: `{}`",
                        rel.display()
                    ),
                    "keep _generated.example text-only curated evidence artifacts",
                    Some(rel),
                ));
            }
            if rel.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let file_name = rel
                    .file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or_default()
                    .to_string();
                let suffix_allowed = file_name.ends_with("-report.json")
                    || file_name.ends_with("-index.json")
                    || file_name.ends_with(".example.json")
                    || matches!(
                        file_name.as_str(),
                        "ALLOWLIST.json"
                            | "ops-ledger.json"
                            | "ops-index.json"
                            | "ops-evidence-bundle.json"
                            | "scorecard.json"
                            | "control-plane-surface-list.json"
                    );
                if !suffix_allowed {
                    violations.push(violation(
                        "OPS_EVIDENCE_FILENAME_SUFFIX_POLICY_VIOLATION",
                        format!(
                            "curated evidence json filename does not match suffix policy: `{}`",
                            rel.display()
                        ),
                        "use -report.json, -index.json, .example.json, or an approved canonical evidence filename",
                        Some(rel),
                    ));
                }
                let text =
                    fs::read_to_string(&file).map_err(|err| CheckError::Failed(err.to_string()))?;
                let json: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|err| CheckError::Failed(err.to_string()))?;
                if json.get("schema_version").is_none() {
                    violations.push(violation(
                        "OPS_EVIDENCE_SCHEMA_VERSION_MISSING",
                        format!(
                            "curated evidence json `{}` must include schema_version",
                            rel.display()
                        ),
                        "add schema_version to curated evidence json artifact",
                        Some(rel),
                    ));
                }
                if json.get("generated_by").is_none() {
                    violations.push(violation(
                        "OPS_EVIDENCE_GENERATED_BY_MISSING",
                        format!(
                            "curated evidence json `{}` must include generated_by",
                            rel.display()
                        ),
                        "add generated_by to curated evidence json artifact",
                        Some(rel),
                    ));
                }
            }
        }
        for allowlisted in &allowlisted_files {
            let rel = Path::new(allowlisted);
            if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_STALE_ENTRY",
                    format!(
                        "allowlist entry points to missing curated artifact `{}`",
                        rel.display()
                    ),
                    "remove stale entry from ALLOWLIST.json or restore the artifact",
                    Some(allowlist_rel),
                ));
            }
            if !seen_files.contains(allowlisted) {
                violations.push(violation(
                    "OPS_EVIDENCE_ALLOWLIST_UNUSED_ENTRY",
                    format!(
                        "allowlist entry does not match a committed curated artifact `{}`",
                        rel.display()
                    ),
                    "keep ALLOWLIST.json aligned with committed files in ops/_generated.example",
                    Some(allowlist_rel),
                ));
            }
        }
    }

    let bundle_text = fs::read_to_string(ctx.repo_root.join(bundle_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let bundle_json: serde_json::Value =
        serde_json::from_str(&bundle_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "release",
        "status",
        "generated_by",
        "generated_from",
        "hashes",
        "gates",
        "pin_freeze_status",
    ] {
        if bundle_json.get(key).is_none() {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_REQUIRED_KEY_MISSING",
                format!("evidence bundle missing required key `{key}`"),
                "populate required evidence bundle key",
                Some(bundle_rel),
            ));
        }
    }
    if bundle_json
        .get("generated_from")
        .and_then(|v| v.as_str())
        != Some(evidence_checklist_rel.to_str().unwrap_or_default())
    {
        violations.push(violation(
            "OPS_EVIDENCE_BUNDLE_LINEAGE_SOURCE_INVALID",
            "release evidence bundle generated_from must reference ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md".to_string(),
            "set release-evidence-bundle.json generated_from to the evidence completeness checklist contract",
            Some(bundle_rel),
        ));
    }
    if let Some(bundle_paths) = bundle_json.get("bundle_paths").and_then(|v| v.as_array()) {
        let bundle_paths = bundle_paths
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<BTreeSet<_>>();
        for required in [
            "ops/report/generated/readiness-score.json",
            "ops/report/generated/historical-comparison.json",
        ] {
            if !bundle_paths.contains(required) {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_COMPLETENESS_PATH_MISSING",
                    format!("release evidence bundle missing required bundle_paths entry `{required}`"),
                    "include readiness and historical comparison artifacts in release-evidence-bundle.json bundle_paths",
                    Some(bundle_rel),
                ));
            }
        }
    }

    if let Some(schema_index) = bundle_json
        .get("hashes")
        .and_then(|v| v.get("schema_index"))
        .and_then(|v| v.as_object())
    {
        let Some(path) = schema_index.get("path").and_then(|v| v.as_str()) else {
            return Ok(violations);
        };
        let Some(sha) = schema_index.get("sha256").and_then(|v| v.as_str()) else {
            return Ok(violations);
        };
        let path_rel = Path::new(path);
        if !ctx.adapters.fs.exists(ctx.repo_root, path_rel) {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_SCHEMA_INDEX_PATH_MISSING",
                format!("schema index path in evidence bundle does not exist: `{path}`"),
                "fix hashes.schema_index.path in evidence bundle",
                Some(bundle_rel),
            ));
        } else {
            let actual_sha = sha256_hex(&ctx.repo_root.join(path_rel))?;
            if actual_sha != sha {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_SCHEMA_INDEX_HASH_DRIFT",
                    "schema index hash in evidence bundle is stale".to_string(),
                    "refresh hashes.schema_index.sha256 in ops-evidence-bundle.json",
                    Some(bundle_rel),
                ));
            }
        }
    }
    if let Some(inventory_hashes) = bundle_json
        .get("hashes")
        .and_then(|v| v.get("inventory"))
        .and_then(|v| v.as_array())
    {
        let mut paths = Vec::new();
        for entry in inventory_hashes {
            let path = entry
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let sha = entry
                .get("sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !path.starts_with("ops/") {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_PATH_INVALID",
                    format!("inventory hash entry path must live under ops/: `{path}`"),
                    "set hashes.inventory[].path to canonical ops paths only",
                    Some(bundle_rel),
                ));
            }
            if !sha.chars().all(|c| c.is_ascii_hexdigit()) || sha.len() != 64 {
                violations.push(violation(
                    "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_INVALID",
                    format!("inventory hash entry must be 64 lowercase hex chars for `{path}`"),
                    "refresh hashes.inventory sha256 values from deterministic artifact hashing",
                    Some(bundle_rel),
                ));
            }
            paths.push(path);
        }
        let mut sorted = paths.clone();
        sorted.sort();
        if paths != sorted {
            violations.push(violation(
                "OPS_EVIDENCE_BUNDLE_INVENTORY_HASH_ORDER_NONDETERMINISTIC",
                "hashes.inventory must be sorted by path for deterministic output".to_string(),
                "sort hashes.inventory entries lexicographically by path",
                Some(bundle_rel),
            ));
        }
    }

    let schema_drift_text = fs::read_to_string(ctx.repo_root.join(schema_drift_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let schema_drift_json: serde_json::Value = serde_json::from_str(&schema_drift_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "generated_by",
        "status",
        "summary",
        "drift",
    ] {
        if schema_drift_json.get(key).is_none() {
            violations.push(violation(
                "OPS_SCHEMA_DRIFT_REPORT_INVALID",
                format!("schema drift report is missing required key `{key}`"),
                "populate schema drift report with required governance keys",
                Some(schema_drift_rel),
            ));
        }
    }

    let readiness_score_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ctx.repo_root.join(readiness_score_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?,
    )
    .map_err(|err| CheckError::Failed(err.to_string()))?;
    let historical_comparison_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ctx.repo_root.join(historical_comparison_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?,
    )
    .map_err(|err| CheckError::Failed(err.to_string()))?;
    let release_report_bundle_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ctx.repo_root.join(release_bundle_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?,
    )
    .map_err(|err| CheckError::Failed(err.to_string()))?;
    if historical_comparison_json
        .get("status")
        .and_then(|v| v.as_str())
        == Some("regressed")
        && release_report_bundle_json
            .get("status")
            .and_then(|v| v.as_str())
            == Some("ready")
    {
        violations.push(violation(
            "OPS_EVIDENCE_READINESS_HISTORICAL_STATUS_CONFLICT",
            "release-evidence-bundle.json status cannot be `ready` when historical-comparison.json status is `regressed`".to_string(),
            "block release readiness or refresh historical comparison evidence before marking release bundle ready",
            Some(release_bundle_rel),
        ));
    }
    if readiness_score_json.get("generated_by").is_none() {
        violations.push(violation(
            "OPS_EVIDENCE_READINESS_SCORE_LINEAGE_MISSING",
            "readiness-score.json must include generated_by for evidence lineage".to_string(),
            "add generated_by metadata to readiness-score.json",
            Some(readiness_score_rel),
        ));
    }

    let evidence_gap_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ctx.repo_root.join(evidence_gap_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?,
    )
    .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "generated_by",
        "generated_from",
        "status",
        "summary",
        "gaps",
    ] {
        if evidence_gap_json.get(key).is_none() {
            violations.push(violation(
                "OPS_EVIDENCE_GAP_REPORT_INVALID",
                format!("evidence gap report is missing required key `{key}`"),
                "populate evidence-gap-report.json with required completeness fields",
                Some(evidence_gap_rel),
            ));
        }
    }
    if evidence_gap_json
        .get("generated_from")
        .and_then(|v| v.as_str())
        != Some(evidence_checklist_rel.to_str().unwrap_or_default())
    {
        violations.push(violation(
            "OPS_EVIDENCE_GAP_REPORT_LINEAGE_SOURCE_INVALID",
            "evidence-gap-report.json generated_from must reference ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md".to_string(),
            "set evidence-gap-report.json generated_from to the evidence completeness checklist contract",
            Some(evidence_gap_rel),
        ));
    }
    if evidence_gap_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_EVIDENCE_GAP_REPORT_BLOCKING",
            "evidence-gap-report.json status is not `pass`".to_string(),
            "resolve missing/stale/lineage evidence gaps and regenerate evidence-gap-report.json",
            Some(evidence_gap_rel),
        ));
    }

    let contract_audit_text = fs::read_to_string(ctx.repo_root.join(contract_audit_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contract_audit_json: serde_json::Value = serde_json::from_str(&contract_audit_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in [
        "schema_version",
        "generated_by",
        "status",
        "summary",
        "contracts",
    ] {
        if contract_audit_json.get(key).is_none() {
            violations.push(violation(
                "OPS_CONTRACT_AUDIT_REPORT_INVALID",
                format!("contract audit report is missing required key `{key}`"),
                "populate contract-audit-report.json with required governance keys",
                Some(contract_audit_rel),
            ));
        }
    }

    let contract_graph_text = fs::read_to_string(ctx.repo_root.join(contract_graph_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let contract_graph_json: serde_json::Value = serde_json::from_str(&contract_graph_text)
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    for key in ["schema_version", "generated_by", "nodes", "edges"] {
        if contract_graph_json.get(key).is_none() {
            violations.push(violation(
                "OPS_CONTRACT_DEPENDENCY_GRAPH_INVALID",
                format!("contract dependency graph is missing required key `{key}`"),
                "populate contract-dependency-graph.json with nodes and dependency edges",
                Some(contract_graph_rel),
            ));
        }
    }

    let gates_text = fs::read_to_string(ctx.repo_root.join(gates_rel))
        .map_err(|err| CheckError::Failed(err.to_string()))?;
    let gates_json: serde_json::Value =
        serde_json::from_str(&gates_text).map_err(|err| CheckError::Failed(err.to_string()))?;
    let expected_gates = gates_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let bundle_gates = bundle_json
        .get("gates")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.get("id").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if expected_gates != bundle_gates {
        violations.push(violation(
            "OPS_EVIDENCE_BUNDLE_GATE_LIST_DRIFT",
            format!(
                "evidence bundle gates mismatch: expected={expected_gates:?} actual={bundle_gates:?}"
            ),
            "synchronize evidence bundle gates list with ops/inventory/gates.json",
            Some(bundle_rel),
        ));
    }

    let generated_root = ctx.repo_root.join("ops/_generated");
    if generated_root.exists() {
        let allowed = BTreeSet::from([
            "ops/_generated/.gitkeep".to_string(),
            "ops/_generated/OWNER.md".to_string(),
            "ops/_generated/README.md".to_string(),
            "ops/_generated/REQUIRED_FILES.md".to_string(),
        ]);
        for file in walk_files(&generated_root) {
            let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
            let rel_str = rel.display().to_string();
            if !allowed.contains(&rel_str) {
                let is_json = rel.extension().and_then(|v| v.to_str()) == Some("json");
                if is_json {
                    violations.push(violation(
                        "OPS_GENERATED_RUNTIME_JSON_COMMITTED_FORBIDDEN",
                        format!(
                            "runtime json evidence must not be committed under ops/_generated: `{}`",
                            rel.display()
                        ),
                        "delete committed runtime json and regenerate into runtime-only ignored outputs",
                        Some(rel),
                    ));
                }
                violations.push(violation(
                    "OPS_GENERATED_DIRECTORY_COMMITTED_EVIDENCE_FORBIDDEN",
                    format!("ops/_generated contains unexpected committed file `{}`", rel.display()),
                    "keep ops/_generated to marker docs only; store curated evidence under ops/_generated.example",
                    Some(rel),
                ));
            }
        }
    }

    Ok(violations)
}
