pub(super) fn check_ops_domain_contract_structure(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let contract_files = [
        "ops/datasets/CONTRACT.md",
        "ops/e2e/CONTRACT.md",
        "ops/env/CONTRACT.md",
        "ops/inventory/CONTRACT.md",
        "ops/k8s/CONTRACT.md",
        "ops/load/CONTRACT.md",
        "ops/observe/CONTRACT.md",
        "ops/report/CONTRACT.md",
        "ops/schema/CONTRACT.md",
        "ops/stack/CONTRACT.md",
    ];
    let mut violations = Vec::new();
    for rel_str in contract_files {
        let rel = Path::new(rel_str);
        if !ctx.adapters.fs.exists(ctx.repo_root, rel) {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_MISSING",
                format!("missing domain contract `{}`", rel.display()),
                "add missing domain CONTRACT.md file",
                Some(rel),
            ));
            continue;
        }
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        if !text.contains("- contract_version: `") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_VERSION_METADATA_MISSING",
                format!(
                    "domain contract `{}` must include `- contract_version: ` metadata",
                    rel.display()
                ),
                "add explicit contract_version metadata in domain CONTRACT.md header",
                Some(rel),
            ));
        }
        let taxonomy = text
            .lines()
            .find_map(|line| {
                let trimmed = line.trim();
                trimmed
                    .strip_prefix("- contract_taxonomy: `")
                    .and_then(|value| value.strip_suffix('`'))
            })
            .unwrap_or_default()
            .to_string();
        if taxonomy.is_empty() {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_TAXONOMY_METADATA_MISSING",
                format!(
                    "domain contract `{}` must include `- contract_taxonomy: ` metadata",
                    rel.display()
                ),
                "set contract_taxonomy to structural, behavioral, or hybrid",
                Some(rel),
            ));
        } else if !matches!(taxonomy.as_str(), "structural" | "behavioral" | "hybrid") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_TAXONOMY_INVALID",
                format!(
                    "domain contract `{}` has invalid contract_taxonomy `{taxonomy}`",
                    rel.display()
                ),
                "use one of: structural, behavioral, hybrid",
                Some(rel),
            ));
        }
        if !text.contains("## Contract Taxonomy") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_TAXONOMY_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Contract Taxonomy`",
                    rel.display()
                ),
                "add structural/behavioral taxonomy section",
                Some(rel),
            ));
        }
        if !text.contains("## Authored vs Generated") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_AUTHORED_GENERATED_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Authored vs Generated`",
                    rel.display()
                ),
                "add an authored-vs-generated table with explicit file paths",
                Some(rel),
            ));
        }
        if !text.contains("## Invariants") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_INVARIANTS_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Invariants`",
                    rel.display()
                ),
                "add an invariants section with explicit, enforceable rules",
                Some(rel),
            ));
            continue;
        }
        if !text.contains("## Enforcement Links") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_ENFORCEMENT_LINKS_MISSING",
                format!(
                    "domain contract `{}` must include `## Enforcement Links`",
                    rel.display()
                ),
                "add enforcement links section that references concrete check ids",
                Some(rel),
            ));
        }
        if !text.contains("## Runtime Evidence Mapping") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_RUNTIME_EVIDENCE_SECTION_MISSING",
                format!(
                    "domain contract `{}` must include `## Runtime Evidence Mapping`",
                    rel.display()
                ),
                "map contract invariants to concrete runtime/generated evidence artifacts",
                Some(rel),
            ));
        }
        if text.contains("locked") || text.contains("Locked") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_STALE_LOCKED_REFERENCE",
                format!(
                    "domain contract `{}` contains stale `locked` wording",
                    rel.display()
                ),
                "remove stale locked-list language from authored domain contracts",
                Some(rel),
            ));
        }
        if !text.contains("checks_") {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_ENFORCEMENT_LINK_EMPTY",
                format!(
                    "domain contract `{}` must reference at least one concrete check id",
                    rel.display()
                ),
                "add at least one `checks_*` identifier under enforcement links",
                Some(rel),
            ));
        }
        let mut in_invariants = false;
        let mut invariant_count = 0usize;
        for line in text.lines() {
            if line.starts_with("## ") {
                in_invariants = line == "## Invariants";
                continue;
            }
            if in_invariants {
                let trimmed = line.trim_start();
                if trimmed.starts_with("- ") {
                    invariant_count += 1;
                }
            }
        }
        if invariant_count < 8 {
            violations.push(violation(
                "OPS_DOMAIN_CONTRACT_INVARIANT_COUNT_TOO_LOW",
                format!(
                    "domain contract `{}` must define at least 8 invariants; found {}",
                    rel.display(),
                    invariant_count
                ),
                "add concrete invariants until the minimum count is satisfied",
                Some(rel),
            ));
        }
    }
    let coverage_rel = Path::new("ops/_generated.example/contract-coverage-report.json");
    if !ctx.adapters.fs.exists(ctx.repo_root, coverage_rel) {
        violations.push(violation(
            "OPS_CONTRACT_COVERAGE_REPORT_MISSING",
            format!(
                "missing contract coverage report `{}`",
                coverage_rel.display()
            ),
            "generate and commit contract coverage report artifact",
            Some(coverage_rel),
        ));
    } else {
        let coverage_text = fs::read_to_string(ctx.repo_root.join(coverage_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        let coverage_json: serde_json::Value = serde_json::from_str(&coverage_text)
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for key in ["schema_version", "generated_by", "contracts"] {
            if coverage_json.get(key).is_none() {
                violations.push(violation(
                    "OPS_CONTRACT_COVERAGE_REPORT_INVALID",
                    format!(
                        "contract coverage report `{}` is missing `{key}`",
                        coverage_rel.display()
                    ),
                    "include schema_version, generated_by, and contracts fields in coverage report",
                    Some(coverage_rel),
                ));
            }
        }
        let contracts = coverage_json
            .get("contracts")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if contracts.is_empty() {
            violations.push(violation(
                "OPS_CONTRACT_COVERAGE_EMPTY",
                "contract coverage report has no contracts entries".to_string(),
                "populate contract-coverage-report.json with domain contract entries",
                Some(coverage_rel),
            ));
        } else {
            let covered = contracts
                .iter()
                .filter(|entry| {
                    entry
                        .get("authored_vs_generated")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        && entry
                            .get("invariants")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0)
                            >= 8
                        && entry
                            .get("enforcement_links")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0)
                            >= 1
                })
                .count();
            let threshold = 80usize;
            let coverage_percent = covered * 100 / contracts.len();
            if coverage_percent < threshold {
                violations.push(violation(
                    "OPS_CONTRACT_COVERAGE_THRESHOLD_NOT_MET",
                    format!(
                        "contract coverage threshold not met: {}% < {}%",
                        coverage_percent, threshold
                    ),
                    "raise contract coverage evidence to at least 80% before merge",
                    Some(coverage_rel),
                ));
            }
        }
    }
    Ok(violations)
}

