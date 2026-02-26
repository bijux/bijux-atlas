pub(super) fn check_ops_domain_contract_structure(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let template_contract_rel = Path::new("ops/DOMAIN_DOCUMENT_TEMPLATE_CONTRACT.md");
    let template_hash_rel = Path::new("ops/_generated.example/domain-document-template-contract.hash.json");
    let template_required_markers = [
        "## Domain CONTRACT.md Required Metadata",
        "## Domain CONTRACT.md Required Sections",
        "## Domain README.md Required Metadata",
        "## generated/README.md Required Metadata",
        "checks_ops_domain_contract_structure",
    ];
    let domain_roots = [
        "ops/datasets",
        "ops/e2e",
        "ops/env",
        "ops/inventory",
        "ops/k8s",
        "ops/load",
        "ops/observe",
        "ops/report",
        "ops/schema",
        "ops/stack",
    ];
    let mut violations = Vec::new();
    if !ctx.adapters.fs.exists(ctx.repo_root, template_contract_rel) {
        violations.push(violation(
            "OPS_DOMAIN_DOCUMENT_TEMPLATE_CONTRACT_MISSING",
            "missing canonical domain document template contract `ops/DOMAIN_DOCUMENT_TEMPLATE_CONTRACT.md`"
                .to_string(),
            "add a canonical domain document template contract consumed by checks_ops_domain_contract_structure",
            Some(template_contract_rel),
        ));
    } else {
        let template_text = fs::read_to_string(ctx.repo_root.join(template_contract_rel))
            .map_err(|err| CheckError::Failed(err.to_string()))?;
        for marker in template_required_markers {
            if !template_text.contains(marker) {
                violations.push(violation(
                    "OPS_DOMAIN_DOCUMENT_TEMPLATE_CONTRACT_INCOMPLETE",
                    format!(
                        "domain document template contract is missing required marker `{marker}`"
                    ),
                    "complete the canonical template contract sections and enforcement linkage",
                    Some(template_contract_rel),
                ));
            }
        }
        if !ctx.adapters.fs.exists(ctx.repo_root, template_hash_rel) {
            violations.push(violation(
                "OPS_DOMAIN_DOCUMENT_TEMPLATE_HASH_MISSING",
                "missing domain document template hash artifact `ops/_generated.example/domain-document-template-contract.hash.json`".to_string(),
                "commit a deterministic template hash artifact for domain document template drift checks",
                Some(template_hash_rel),
            ));
        } else {
            let hash_text = fs::read_to_string(ctx.repo_root.join(template_hash_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let hash_json: serde_json::Value = serde_json::from_str(&hash_text)
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            let expected_path = hash_json
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if expected_path != template_contract_rel.display().to_string() {
                violations.push(violation(
                    "OPS_DOMAIN_DOCUMENT_TEMPLATE_HASH_PATH_INVALID",
                    "domain template hash artifact path must target ops/DOMAIN_DOCUMENT_TEMPLATE_CONTRACT.md"
                        .to_string(),
                    "set hash artifact `path` to the canonical template contract path",
                    Some(template_hash_rel),
                ));
            }
            let expected_sha = hash_json
                .get("sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if expected_sha.is_empty() {
                violations.push(violation(
                    "OPS_DOMAIN_DOCUMENT_TEMPLATE_HASH_EMPTY",
                    "domain template hash artifact must include non-empty sha256".to_string(),
                    "populate sha256 with the template contract digest",
                    Some(template_hash_rel),
                ));
            } else {
                let actual_sha = sha256_hex(&ctx.repo_root.join(template_contract_rel))?;
                if expected_sha != actual_sha {
                    violations.push(violation(
                        "OPS_DOMAIN_DOCUMENT_TEMPLATE_HASH_DRIFT",
                        format!(
                            "domain template contract hash drift: expected {expected_sha} actual {actual_sha}"
                        ),
                        "refresh ops/_generated.example/domain-document-template-contract.hash.json after template changes",
                        Some(template_hash_rel),
                    ));
                }
            }
        }
    }
    for domain_root in domain_roots {
        let contract_rel_string = format!("{domain_root}/CONTRACT.md");
        let rel = Path::new(&contract_rel_string);
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
        for required_header in ["- Owner:", "- Purpose:", "- Consumers:"] {
            if !text.contains(required_header) {
                violations.push(violation(
                    "OPS_DOMAIN_DOC_HEADER_METADATA_MISSING",
                    format!(
                        "domain contract `{}` must include header metadata line `{required_header}`",
                        rel.display()
                    ),
                    "add Owner, Purpose, and Consumers metadata lines near the top of the domain contract",
                    Some(rel),
                ));
            }
        }
        let consumer_line = text
            .lines()
            .find(|line| line.trim_start().starts_with("- Consumers:"))
            .unwrap_or_default();
        if consumer_line.trim() == "- Consumers:" {
            violations.push(violation(
                "OPS_DOMAIN_DOC_CONSUMERS_METADATA_EMPTY",
                format!("domain contract `{}` must declare at least one consumer", rel.display()),
                "list the enforcing checks or runtime commands under Consumers metadata",
                Some(rel),
            ));
        }
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

        let readme_rel_string = format!("{domain_root}/README.md");
        let readme_rel = Path::new(&readme_rel_string);
        if !ctx.adapters.fs.exists(ctx.repo_root, readme_rel) {
            violations.push(violation(
                "OPS_DOMAIN_README_MISSING",
                format!("missing domain README `{}`", readme_rel.display()),
                "add missing domain README.md file",
                Some(readme_rel),
            ));
        } else {
            let readme_text = fs::read_to_string(ctx.repo_root.join(readme_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            for required_header in ["- Owner:", "- Purpose:", "- Consumers:"] {
                if !readme_text.contains(required_header) {
                    violations.push(violation(
                        "OPS_DOMAIN_DOC_HEADER_METADATA_MISSING",
                        format!(
                            "domain README `{}` must include header metadata line `{required_header}`",
                            readme_rel.display()
                        ),
                        "add Owner, Purpose, and Consumers metadata lines near the top of the domain README",
                        Some(readme_rel),
                    ));
                }
            }
            let consumer_line = readme_text
                .lines()
                .find(|line| line.trim_start().starts_with("- Consumers:"))
                .unwrap_or_default();
            if consumer_line.trim() == "- Consumers:" {
                violations.push(violation(
                    "OPS_DOMAIN_DOC_CONSUMERS_METADATA_EMPTY",
                    format!("domain README `{}` must declare at least one consumer", readme_rel.display()),
                    "list the runtime command surface or checks that consume this README",
                    Some(readme_rel),
                ));
            }
        }

        let generated_readme_rel_string = format!("{domain_root}/generated/README.md");
        let generated_readme_rel = Path::new(&generated_readme_rel_string);
        if ctx.adapters.fs.exists(ctx.repo_root, generated_readme_rel) {
            let generated_readme_text = fs::read_to_string(ctx.repo_root.join(generated_readme_rel))
                .map_err(|err| CheckError::Failed(err.to_string()))?;
            for required_line in ["- Producer:", "- Regenerate:"] {
                if !generated_readme_text.contains(required_line) {
                    violations.push(violation(
                        "OPS_GENERATED_README_PRODUCER_METADATA_MISSING",
                        format!(
                            "generated README `{}` must include `{required_line}` metadata",
                            generated_readme_rel.display()
                        ),
                        "document producer command and regeneration command in generated/README.md",
                        Some(generated_readme_rel),
                    ));
                }
            }
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
