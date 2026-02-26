fn validate_ops_authority_tiers_and_doc_necessity(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let ops_docs_contract_rel = Path::new("docs/operations/ops-docs-contract.md");
    let ops_docs_contract_text = fs::read_to_string(ctx.repo_root.join(ops_docs_contract_rel)).map_err(
        |err| CheckError::Failed(format!("read {}: {err}", ops_docs_contract_rel.display())),
    )?;
    for required in [
        "- Owner:",
        "- Tier:",
        "- Audience:",
        "- Source-of-truth:",
        "## Authority Tiers",
        "tier0-machine",
        "tier1-normative",
        "tier2",
        "## Authority Exceptions",
        "authority-tier-exceptions.json",
        "## Enforcement",
    ] {
        if !ops_docs_contract_text.contains(required) {
            violations.push(violation(
                "OPS_DOCS_AUTHORITY_CONTRACT_INCOMPLETE",
                format!(
                    "docs authority contract `{}` is missing `{required}`",
                    ops_docs_contract_rel.display()
                ),
                "complete docs/operations/ops-docs-contract.md with tier definitions and enforcement links",
                Some(ops_docs_contract_rel),
            ));
        }
    }

    for (rel, required_snippets) in [
        (
            Path::new("docs/operations/reference/commands.md"),
            ["- Tier: `generated`", "## bijux-dev-atlas", "## bijux-dev-atlas ops"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/ops-surface.md"),
            ["- Tier: `generated`", "ops/inventory/surfaces.json", "ops/_generated.example/control-plane.snapshot.md"]
                .as_slice(),
        ),
        (
            Path::new("docs/operations/entrypoints.md"),
            ["- Tier: `tier2`", "## Canonical Entrypoints", "reference/commands.md", "reference/ops-surface.md"]
                .as_slice(),
        ),
        (
            Path::new("docs/operations/reference/tools.md"),
            ["- Tier: `generated`", "ops/inventory/tools.toml", "## Tools"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/toolchain.md"),
            ["- Tier: `generated`", "ops/inventory/toolchain.json", "## GitHub Actions Pins"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/pins.md"),
            ["- Tier: `generated`", "ops/inventory/pins.yaml", "## Pins"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/gates.md"),
            ["- Tier: `generated`", "ops/inventory/gates.json", "## Gates"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/drills.md"),
            ["- Tier: `generated`", "ops/inventory/drills.json", "## Drills"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/schema-index.md"),
            ["- Tier: `generated`", "ops/schema/generated/schema-index.md", "## Canonical Source"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/evidence-model.md"),
            ["- Tier: `generated`", "evidence-levels.schema.json", "release-evidence-bundle.schema.json"].as_slice(),
        ),
        (
            Path::new("docs/operations/reference/what-breaks-if-removed.md"),
            ["- Tier: `generated`", "ops/_generated.example/what-breaks-if-removed-report.json", "## Removal Impact Targets"]
                .as_slice(),
        ),
        (
            Path::new("docs/operations/DOCS_CONVERGENCE_POLICY.md"),
            ["- Tier: `tier2`", "## Convergence Rule", "## Deletion Rule", "checks_ops_docs_governance"].as_slice(),
        ),
        (
            Path::new("docs/operations/DOCS_STRUCTURE_FREEZE.md"),
            ["- Tier: `tier2`", "## Version", "v0.1", "## Change Control"].as_slice(),
        ),
    ] {
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
        for snippet in required_snippets {
            if !text.contains(snippet) {
                violations.push(violation(
                    "OPS_DOCS_REFERENCE_ENTRYPOINT_INCOMPLETE",
                    format!("documentation page `{}` is missing `{snippet}`", rel.display()),
                    "restore the generated reference/entrypoints docs and required links",
                    Some(rel),
                ));
            }
        }
    }

    let external_allowlist_rel = Path::new("docs/operations/external-link-allowlist.json");
    let external_allowlist_text = fs::read_to_string(ctx.repo_root.join(external_allowlist_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", external_allowlist_rel.display())))?;
    let external_allowlist_json: serde_json::Value = serde_json::from_str(&external_allowlist_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", external_allowlist_rel.display())))?;
    let mut allowed_external_urls = std::collections::HashSet::new();
    for entry in external_allowlist_json
        .get("entries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
    {
        let url = entry.get("url").and_then(|v| v.as_str()).unwrap_or_default();
        let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or_default();
        let expires_on = entry.get("expires_on").and_then(|v| v.as_str()).unwrap_or_default();
        if url.is_empty() || reason.trim().is_empty() || expires_on.is_empty() {
            violations.push(violation(
                "OPS_DOC_EXTERNAL_LINK_ALLOWLIST_INCOMPLETE",
                format!("external link allowlist `{}` entries must include url/reason/expires_on", external_allowlist_rel.display()),
                "complete each allowlist entry with url, reason, and expires_on",
                Some(external_allowlist_rel),
            ));
            continue;
        }
        let parts: Vec<&str> = expires_on.split('-').collect();
        let is_date = parts.len() == 3
            && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
            && parts[0].len() == 4
            && parts[1].len() == 2
            && parts[2].len() == 2;
        if !is_date {
            violations.push(violation(
                "OPS_DOC_EXTERNAL_LINK_ALLOWLIST_EXPIRY_INVALID",
                format!("external link allowlist entry `{url}` has invalid expires_on `{expires_on}`"),
                "use YYYY-MM-DD expiry dates in docs/operations/external-link-allowlist.json",
                Some(external_allowlist_rel),
            ));
        }
        allowed_external_urls.insert(url.to_string());
    }

    let docs_delete_sim_rel = Path::new("docs/_generated/docs-what-breaks-if-removed-report.json");
    let docs_delete_sim_text = fs::read_to_string(ctx.repo_root.join(docs_delete_sim_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", docs_delete_sim_rel.display())))?;
    let docs_delete_sim_json: serde_json::Value = serde_json::from_str(&docs_delete_sim_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", docs_delete_sim_rel.display())))?;
    if docs_delete_sim_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_DOCS_DELETION_IMPACT_REPORT_BLOCKING",
            format!("docs deletion-impact report `{}` status is not `pass`", docs_delete_sim_rel.display()),
            "refresh docs-what-breaks-if-removed-report.json and resolve blocking impacts",
            Some(docs_delete_sim_rel),
        ));
    }
    if docs_delete_sim_json
        .get("targets")
        .and_then(|v| v.as_array())
        .map(|a| a.is_empty())
        .unwrap_or(true)
    {
        violations.push(violation(
            "OPS_DOCS_DELETION_IMPACT_REPORT_EMPTY",
            format!("docs deletion-impact report `{}` must include targets", docs_delete_sim_rel.display()),
            "add docs deletion-impact targets and consumers to the curated report",
            Some(docs_delete_sim_rel),
        ));
    }

    let authority_tiers_rel = Path::new("ops/AUTHORITY_TIERS.md");
    let authority_tiers_text = fs::read_to_string(ctx.repo_root.join(authority_tiers_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", authority_tiers_rel.display())))?;
    for required in [
        "- Authority Tier:",
        "- Audience:",
        "## Tiers",
        "## Tier Rules",
        "## Audience Tags",
        "tier0-machine",
        "tier1-normative",
        "tier2",
        "generated",
    ] {
        if !authority_tiers_text.contains(required) {
            violations.push(violation(
                "OPS_DOC_AUTHORITY_TIERS_CONTRACT_INCOMPLETE",
                format!(
                    "authority tiers contract `{}` is missing `{required}`",
                    authority_tiers_rel.display()
                ),
                "complete ops/AUTHORITY_TIERS.md with required metadata and tier definitions",
                Some(authority_tiers_rel),
            ));
        }
    }

    let doc_necessity_rel = Path::new("ops/DOC_NECESSITY_CHECKLIST.md");
    let doc_necessity_text = fs::read_to_string(ctx.repo_root.join(doc_necessity_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", doc_necessity_rel.display())))?;
    for required in [
        "- Authority Tier:",
        "- Audience:",
        "## Checklist",
        "single clear consumer",
        "duplicating semantics",
        "why deletion would break",
    ] {
        if !doc_necessity_text.contains(required) {
            violations.push(violation(
                "OPS_DOC_NECESSITY_CHECKLIST_INCOMPLETE",
                format!(
                    "doc necessity checklist `{}` is missing `{required}`",
                    doc_necessity_rel.display()
                ),
                "complete ops/DOC_NECESSITY_CHECKLIST.md with required checklist criteria",
                Some(doc_necessity_rel),
            ));
        }
    }

    let valid_tiers = ["machine", "explanatory", "generated", "tier0-machine", "tier1-normative", "tier2"];
    let valid_audiences = ["contributors", "operators", "reviewers", "mixed"];
    let allowed_tier1_root_docs = [
        "ops/README.md",
        "ops/CONTRACT.md",
        "ops/SSOT.md",
        "ops/ERRORS.md",
        "ops/DRIFT.md",
        "ops/NAMING.md",
        "ops/ARTIFACTS.md",
        "ops/GENERATED_LIFECYCLE.md",
        "ops/AUTHORITY_TIERS.md",
        "ops/CONTROL_PLANE.md",
        "ops/DIRECTORY_BUDGET_POLICY.md",
        "ops/DOMAIN_DOCUMENT_TEMPLATE_CONTRACT.md",
        "ops/TIER1_ROOT_SURFACE.md",
    ];
    let mut tier1_root_doc_count = 0usize;
    for file in walk_files(&ctx.repo_root.join("ops")) {
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        if rel.components().count() != 2 {
            continue;
        }
        if file.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&file)
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
        let tier = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Authority Tier: `"))
            .and_then(|v| v.strip_suffix('`'))
            .unwrap_or_default()
            .to_string();
        let audience = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Audience: `"))
            .and_then(|v| v.strip_suffix('`'))
            .unwrap_or_default()
            .to_string();
        if tier.is_empty() {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUTHORITY_TIER_MISSING",
                format!("top-level ops doc `{}` must declare `- Authority Tier:`", rel.display()),
                "add Authority Tier metadata (`machine`, `explanatory`, or `generated`) to every top-level ops/*.md doc",
                Some(rel),
            ));
            continue;
        }
        if !valid_tiers.contains(&tier.as_str()) {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUTHORITY_TIER_INVALID",
                format!(
                    "top-level ops doc `{}` has invalid Authority Tier `{}`",
                    rel.display(),
                    tier
                ),
                "use one of: machine, explanatory, generated",
                Some(rel),
            ));
        }
        if audience.is_empty() {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUDIENCE_MISSING",
                format!("top-level ops doc `{}` must declare `- Audience:`", rel.display()),
                "add Audience metadata (`contributors`, `operators`, `reviewers`, or `mixed`) to every top-level ops/*.md doc",
                Some(rel),
            ));
        } else if !valid_audiences.contains(&audience.as_str()) {
            violations.push(violation(
                "OPS_TOP_LEVEL_DOC_AUDIENCE_INVALID",
                format!(
                    "top-level ops doc `{}` has invalid Audience `{}`",
                    rel.display(),
                    audience
                ),
                "use one of: contributors, operators, reviewers, mixed",
                Some(rel),
            ));
        }

        if tier == "machine" || tier == "tier0-machine" || tier == "tier1-normative" {
            tier1_root_doc_count += 1;
            let rel_s = rel.display().to_string();
            if !allowed_tier1_root_docs.contains(&rel_s.as_str()) {
                violations.push(violation(
                    "OPS_TIER1_ROOT_DOC_NOT_ALLOWLISTED",
                    format!(
                        "top-level ops doc `{}` is Tier-1 but not in the allowed Tier-1 root surface",
                        rel.display()
                    ),
                    "reclassify to tier2/generated or add to ops/TIER1_ROOT_SURFACE.md with justification",
                    Some(rel),
                ));
            }
        }

        if tier == "explanatory" || tier == "tier2" {
            for forbidden_header in ["## Invariants", "## Contract", "## Rules"] {
                if text.contains(forbidden_header) {
                    violations.push(violation(
                        "OPS_EXPLANATORY_DOC_DEFINES_NORMATIVE_SECTION",
                        format!(
                            "explanatory top-level ops doc `{}` contains normative section header `{}`",
                            rel.display(),
                            forbidden_header
                        ),
                        "move normative rules to machine-tier contracts/policies and keep explanatory docs descriptive",
                        Some(rel),
                    ));
                }
            }
        }

        if tier == "machine" || tier == "tier0-machine" || tier == "tier1-normative" {
            for forbidden in ["| Tool |", "| Pin |", "## How to run", "```bash\nmake "] {
                if text.contains(forbidden) {
                    violations.push(violation(
                        "OPS_TIER1_DOC_CONTAINS_WORKFLOW_OR_STRUCTURED_RUNTIME_TRUTH",
                        format!(
                            "normative ops doc `{}` contains disallowed Tier-1 content pattern `{}`",
                            rel.display(), forbidden
                        ),
                        "keep Tier-1 docs minimal and normative; move workflows/tutorial content to docs/operations or machine inventories",
                        Some(rel),
                    ));
                }
            }
        }
    }

    if tier1_root_doc_count > 13 {
        violations.push(violation(
            "OPS_TIER1_ROOT_DOC_COUNT_BUDGET_EXCEEDED",
            format!(
                "Tier-1 root docs exceed budget: found {}, budget=13",
                tier1_root_doc_count
            ),
            "shrink ops root Tier-1 surface or reclassify narrative docs to tier2",
            Some(Path::new("ops/TIER1_ROOT_SURFACE.md")),
        ));
    }

    let exceptions_rel = Path::new("ops/inventory/authority-tier-exceptions.json");
    let exceptions_text = fs::read_to_string(ctx.repo_root.join(exceptions_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", exceptions_rel.display())))?;
    let exceptions_json: serde_json::Value = serde_json::from_str(&exceptions_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", exceptions_rel.display())))?;
    let mut tier2_exceptions: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for entry in exceptions_json
        .get("exceptions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
    {
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or_default();
        let rule = entry.get("rule").and_then(|v| v.as_str()).unwrap_or_default();
        let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or_default();
        let expires_on = entry.get("expires_on").and_then(|v| v.as_str()).unwrap_or_default();
        if path.is_empty() || rule.is_empty() || reason.trim().is_empty() || expires_on.is_empty() {
            violations.push(violation(
                "OPS_DOC_AUTHORITY_EXCEPTION_INCOMPLETE",
                format!("authority tier exception entries in `{}` must include path/rule/reason/expires_on", exceptions_rel.display()),
                "complete every exception with path, rule, reason, and expires_on",
                Some(exceptions_rel),
            ));
            continue;
        }
        let parts: Vec<&str> = expires_on.split('-').collect();
        let is_date = parts.len() == 3
            && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
            && parts[0].len() == 4
            && parts[1].len() == 2
            && parts[2].len() == 2;
        if !is_date {
            violations.push(violation(
                "OPS_DOC_AUTHORITY_EXCEPTION_EXPIRY_INVALID",
                format!("authority tier exception `{path}` has invalid expires_on `{expires_on}` in `{}`", exceptions_rel.display()),
                "use YYYY-MM-DD expiry dates for authority tier exceptions",
                Some(exceptions_rel),
            ));
        }
        tier2_exceptions.insert((path.to_string(), rule.to_string()));
    }

    let valid_docs_tiers = ["tier0-machine", "tier1-normative", "tier2", "generated"];
    for file in walk_files(&ctx.repo_root.join("docs/operations")) {
        if file.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let rel = file.strip_prefix(ctx.repo_root).unwrap_or(file.as_path());
        let text = fs::read_to_string(&file)
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
        let tier = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Tier: `"))
            .and_then(|v| v.strip_suffix('`'))
            .unwrap_or_default()
            .to_string();
        let owner = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Owner: `"))
            .and_then(|v| v.strip_suffix('`'))
            .unwrap_or_default();
        let audience = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Audience: `"))
            .and_then(|v| v.strip_suffix('`'))
            .unwrap_or_default();
        let has_sot = text.lines().any(|line| line.trim().starts_with("- Source-of-truth:"));
        if owner.is_empty() || tier.is_empty() || audience.is_empty() || !has_sot {
            violations.push(violation(
                "OPS_DOCS_PAGE_TIER_HEADER_MISSING",
                format!(
                    "docs operations page `{}` must declare Owner/Tier/Audience/Source-of-truth header metadata",
                    rel.display()
                ),
                "add header metadata lines directly below the title in docs/operations pages",
                Some(rel),
            ));
            continue;
        }
        if !valid_docs_tiers.contains(&tier.as_str()) {
            violations.push(violation(
                "OPS_DOCS_PAGE_TIER_INVALID",
                format!("docs operations page `{}` has invalid tier `{}`", rel.display(), tier),
                "use one of: tier0-machine, tier1-normative, tier2, generated",
                Some(rel),
            ));
        }
        if !valid_audiences.contains(&audience) {
            violations.push(violation(
                "OPS_DOCS_PAGE_AUDIENCE_INVALID",
                format!("docs operations page `{}` has invalid audience `{}`", rel.display(), audience),
                "use one of: contributors, operators, reviewers, mixed",
                Some(rel),
            ));
        }

        if tier == "tier2" {
            let rel_s = rel.display().to_string();
            let has_dir_map = text.contains("| Path |") || text.contains("```\nops/") || text.contains("```text\nops/");
            if has_dir_map && !tier2_exceptions.contains(&(rel_s.clone(), "tier2_no_directory_map".to_string())) {
                violations.push(violation(
                    "OPS_TIER2_DOC_DIRECTORY_MAP_BANNED",
                    format!("tier2 page `{}` contains a directory-map style block/table without exception", rel.display()),
                    "link to authoritative ops surface docs or add a temporary exception with expiry",
                    Some(rel),
                ));
            }
            let has_command_list = text.contains("cargo run -p bijux-dev-atlas -- ops") || text.contains("`make ops-") || text.contains("\n```bash\nmake ops-");
            if has_command_list && !tier2_exceptions.contains(&(rel_s.clone(), "tier2_no_command_list".to_string())) {
                violations.push(violation(
                    "OPS_TIER2_DOC_COMMAND_LIST_BANNED",
                    format!("tier2 page `{}` contains ops command surface examples without exception", rel.display()),
                    "use generated command reference pages or add a temporary exception with expiry",
                    Some(rel),
                ));
            }

            let top_level_docs_page = rel.components().count() == 3;
            let allowed_top_level_command_docs = [
                "docs/operations/entrypoints.md",
                "docs/operations/INDEX.md",
                "docs/operations/reference/commands.md",
                "docs/operations/reference/ops-surface.md",
            ];
            let contains_make_ops = text.contains("make ops-") || text.contains("$ make ops-");
            let contains_cli_ops = text.contains("bijux-dev-atlas ops") || text.contains("bijux dev atlas ops");
            if top_level_docs_page
                && (contains_make_ops || contains_cli_ops)
                && !allowed_top_level_command_docs.contains(&rel_s.as_str())
            {
                violations.push(violation(
                    "OPS_TOP_LEVEL_TIER2_DOC_COMMAND_SURFACE_DUPLICATION",
                    format!(
                        "top-level tier2 docs page `{}` embeds command-surface examples; link to generated references instead",
                        rel.display()
                    ),
                    "move command lists to docs/operations/reference/commands.md or docs/operations/reference/ops-surface.md",
                    Some(rel),
                ));
            }

            let top_level_topic_table_ban = [
                ("tools", "| Tool |", "docs/operations/reference/tools.md"),
                ("toolchain", "## GitHub Actions Pins", "docs/operations/reference/toolchain.md"),
                ("pins", "| Section | Key | Value |", "docs/operations/reference/pins.md"),
                ("gates", "| Gate ID | Category | Action ID | Description |", "docs/operations/reference/gates.md"),
                ("drills", "ops.drill.", "docs/operations/reference/drills.md"),
            ];
            let top_level_ref_pages = [
                "docs/operations/reference/tools.md",
                "docs/operations/reference/toolchain.md",
                "docs/operations/reference/pins.md",
                "docs/operations/reference/gates.md",
                "docs/operations/reference/drills.md",
            ];
            if top_level_docs_page && !top_level_ref_pages.contains(&rel_s.as_str()) {
                for (topic, marker, reference_page) in top_level_topic_table_ban {
                    if text.contains(marker) {
                        violations.push(violation(
                            "OPS_TOP_LEVEL_TIER2_DOC_HAND_EDITED_REFERENCE_TABLE_BANNED",
                            format!(
                                "top-level tier2 docs page `{}` contains hand-edited {} reference content; use `{}`",
                                rel.display(), topic, reference_page
                            ),
                            "replace hand-maintained topic tables/lists with links to generated docs/operations/reference pages",
                            Some(rel),
                        ));
                    }
                }
            }

            if text.contains("edit ops/_generated") || text.contains("edit ops/_generated.example") {
                violations.push(violation(
                    "OPS_DOCS_DIRECT_GENERATED_EDIT_INSTRUCTION_BANNED",
                    format!(
                        "tier2 docs page `{}` instructs editing generated dirs directly",
                        rel.display()
                    ),
                    "describe regeneration commands or authoritative sources instead of manual edits in ops/_generated*",
                    Some(rel),
                ));
            }

            let source_line = text
                .lines()
                .find(|line| line.trim().starts_with("- Source-of-truth:"))
                .unwrap_or_default();
            if !(source_line.contains("ops/") || source_line.contains("docs/operations/reference/")) {
                violations.push(violation(
                    "OPS_TIER2_DOC_SOURCE_LINK_NOT_EXPLICIT",
                    format!(
                        "tier2 docs page `{}` must point Source-of-truth to ops/* or docs/operations/reference/* authoritative sources",
                        rel.display()
                    ),
                    "update Source-of-truth header to reference normative ops sources or generated reference pages",
                    Some(rel),
                ));
            }
        }

        let file_name = rel.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_ascii_lowercase();
        if (file_name.contains("future") || file_name.contains("todo") || file_name.contains("draft"))
            && rel.components().count() >= 3
        {
            violations.push(violation(
                "OPS_DOCS_PLACEHOLDER_PAGE_NAME_BANNED",
                format!("docs operations page uses placeholder naming: `{}`", rel.display()),
                "rename placeholder pages to durable intent-based names or delete them",
                Some(rel),
            ));
        }

        for token in text.split_whitespace() {
            let candidate = token.trim_matches(|c: char| "`()[]<>{},.;'\"".contains(c));
            if !(candidate.starts_with("http://") || candidate.starts_with("https://")) {
                continue;
            }
            if !allowed_external_urls.contains(candidate) {
                violations.push(violation(
                    "OPS_DOC_EXTERNAL_LINK_NOT_ALLOWLISTED",
                    format!(
                        "docs operations page `{}` references external URL `{}` not in docs/operations/external-link-allowlist.json",
                        rel.display(), candidate
                    ),
                    "add the URL to the docs external-link allowlist with reason and expiry or replace it with a local/generated reference",
                    Some(rel),
                ));
            }
        }
    }

    let docs_shrink_rel = Path::new("ops/_generated.example/docs-shrink-report.json");
    let docs_shrink_text = fs::read_to_string(ctx.repo_root.join(docs_shrink_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", docs_shrink_rel.display())))?;
    let docs_shrink_json: serde_json::Value = serde_json::from_str(&docs_shrink_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", docs_shrink_rel.display())))?;
    if docs_shrink_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_DOCS_SHRINK_REPORT_BLOCKING",
            format!("docs shrink report `{}` status is not `pass`", docs_shrink_rel.display()),
            "resolve docs compression budget failures and regenerate docs-shrink-report.json",
            Some(docs_shrink_rel),
        ));
    }
    let max_md_per_dir = docs_shrink_json
        .get("budgets")
        .and_then(|v| v.get("max_markdown_files_per_ops_domain_dir"))
        .and_then(|v| v.as_u64())
        .unwrap_or(18);
    let top_dirs = docs_shrink_json
        .get("top_directories")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if top_dirs.is_empty() {
        violations.push(violation(
            "OPS_DOCS_SHRINK_REPORT_EMPTY",
            format!("docs shrink report `{}` must include top_directories entries", docs_shrink_rel.display()),
            "include top_directories markdown counts for canonical ops domains",
            Some(docs_shrink_rel),
        ));
    }

    for domain in [
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
    ] {
        let domain_dir = ctx.repo_root.join(domain);
        if !domain_dir.exists() {
            continue;
        }
        let md_count = walk_files(&domain_dir)
            .into_iter()
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("md"))
            .count() as u64;
        if md_count > max_md_per_dir {
            violations.push(violation(
                "OPS_DOMAIN_DOC_COUNT_BUDGET_EXCEEDED",
                format!(
                    "domain markdown file count exceeds budget: `{domain}` has {md_count}, budget={max_md_per_dir}"
                ),
                "consolidate docs, generate references, or raise docs-shrink-report budget with justification",
                Some(Path::new(domain)),
            ));
        }
    }

    let docs_dup_rel = Path::new("ops/_generated.example/docs-semantic-duplication-report.json");
    let docs_dup_text = fs::read_to_string(ctx.repo_root.join(docs_dup_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", docs_dup_rel.display())))?;
    let docs_dup_json: serde_json::Value = serde_json::from_str(&docs_dup_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", docs_dup_rel.display())))?;
    if docs_dup_json.get("status").and_then(|v| v.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_DOCS_SEMANTIC_DUPLICATION_REPORT_BLOCKING",
            format!("docs semantic duplication report `{}` status is not `pass`", docs_dup_rel.display()),
            "resolve duplicate-risk docs or regenerate docs-semantic-duplication-report.json",
            Some(docs_dup_rel),
        ));
    }
    let pairs = docs_dup_json.get("pairs").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if pairs.is_empty() {
        violations.push(violation(
            "OPS_DOCS_SEMANTIC_DUPLICATION_REPORT_EMPTY",
            format!("docs semantic duplication report `{}` must include at least one analyzed pair", docs_dup_rel.display()),
            "emit analyzed doc pairs in docs-semantic-duplication-report.json",
            Some(docs_dup_rel),
        ));
    }

    Ok(())
}
