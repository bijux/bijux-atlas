// SPDX-License-Identifier: Apache-2.0

pub(super) fn checks_ops_human_workflow_maturity(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    validate_required_human_workflow_docs(ctx, &mut violations)?;
    validate_drill_ownership_coverage(ctx, &mut violations)?;
    Ok(violations)
}

fn validate_required_human_workflow_docs(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let doc_requirements = [
        (
            "ops/SCHEMA_EVOLUTION_WORKFLOW.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Workflow",
                "## Required Inputs",
                "ops/schema/VERSIONING_POLICY.md",
                "ops/schema/generated/compatibility-lock.json",
                "## Enforcement Links",
            ]
            .as_slice(),
        ),
        (
            "ops/OPS_CHANGE_REVIEW_CHECKLIST.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Checklist",
                "Authority updated",
                "Schema coverage updated",
                "Evidence impact reviewed",
                "## Escalation Conditions",
            ]
            .as_slice(),
        ),
        (
            "ops/BREAKING_CHANGE_TEMPLATE.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Change Summary",
                "## Impact Analysis",
                "## Migration Plan",
                "## Approval",
            ]
            .as_slice(),
        ),
        (
            "ops/OPS_ADR_TEMPLATE.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Title",
                "## Status",
                "## Context",
                "## Decision",
                "## Consequences",
                "## Contract and Check Impact",
                "## Reviewers",
            ]
            .as_slice(),
        ),
        (
            "ops/RELEASE_READINESS_SIGNOFF_CHECKLIST.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Sign-Off Checklist",
                "readiness-score.json",
                "historical-comparison.json",
                "release-evidence-bundle.json",
                "## Required Sign-Off Roles",
            ]
            .as_slice(),
        ),
        (
            "ops/EVIDENCE_SIGNOFF_WORKFLOW.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Workflow",
                "## Required Inputs",
                "## Required Sign-Off Roles",
                "## Enforcement Links",
            ]
            .as_slice(),
        ),
        (
            "docs/operations/REDIRECT_EXPIRY_WORKFLOW.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Workflow",
                "## Required Metadata",
                "Expiry Date",
                "Replacement Path",
                "## Enforcement Links",
            ]
            .as_slice(),
        ),
        (
            "ops/observe/drills/OWNERSHIP.md",
            [
                "- Owner:",
                "- Purpose:",
                "- Consumers:",
                "## Ownership Map",
                "## Enforcement Links",
            ]
            .as_slice(),
        ),
    ];

    for (rel_str, required_snippets) in doc_requirements {
        let rel = Path::new(rel_str);
        let path = ctx.repo_root.join(rel);
        if !path.exists() {
            violations.push(violation(
                "OPS_HUMAN_WORKFLOW_DOC_MISSING",
                format!("missing human workflow contract `{}`", rel.display()),
                "restore the required workflow contract document",
                Some(rel),
            ));
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
        for snippet in required_snippets {
            if !text.contains(snippet) {
                violations.push(violation(
                    "OPS_HUMAN_WORKFLOW_DOC_INCOMPLETE",
                    format!(
                        "workflow contract `{}` is missing required content `{snippet}`",
                        rel.display()
                    ),
                    "complete the workflow contract with the required metadata and sections",
                    Some(rel),
                ));
            }
        }
    }
    Ok(())
}

fn validate_drill_ownership_coverage(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let inventory_drills_rel = Path::new("ops/inventory/drills.json");
    let inventory_drills_text = fs::read_to_string(ctx.repo_root.join(inventory_drills_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", inventory_drills_rel.display())))?;
    let inventory_drills_json: serde_json::Value = serde_json::from_str(&inventory_drills_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", inventory_drills_rel.display())))?;
    let drill_ids = inventory_drills_json
        .get("drills")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();

    let owners_rel = Path::new("ops/inventory/owners.json");
    let owners_text = fs::read_to_string(ctx.repo_root.join(owners_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", owners_rel.display())))?;
    let owners_json: serde_json::Value = serde_json::from_str(&owners_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", owners_rel.display())))?;
    let owner_values = owners_json
        .get("areas")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.values()
                .filter_map(|v| v.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let ownership_rel = Path::new("ops/observe/drills/OWNERSHIP.md");
    let ownership_text = fs::read_to_string(ctx.repo_root.join(ownership_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", ownership_rel.display())))?;

    let mut declared_drills = BTreeSet::new();
    for line in ownership_text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("- `ops.drill.") else {
            continue;
        };
        let Some((id_suffix, owner_part)) = rest.split_once("`: `") else {
            violations.push(violation(
                "OPS_DRILL_OWNERSHIP_FORMAT_INVALID",
                format!(
                    "drill ownership entry has invalid format in `{}`: `{trimmed}`",
                    ownership_rel.display()
                ),
                "use `- `ops.drill.<id>`: `<owner>` format in ops/observe/drills/OWNERSHIP.md",
                Some(ownership_rel),
            ));
            continue;
        };
        let Some(owner) = owner_part.strip_suffix('`') else {
            violations.push(violation(
                "OPS_DRILL_OWNERSHIP_FORMAT_INVALID",
                format!(
                    "drill ownership entry has invalid owner quoting in `{}`: `{trimmed}`",
                    ownership_rel.display()
                ),
                "wrap owner ids in backticks and keep one owner per drill entry",
                Some(ownership_rel),
            ));
            continue;
        };
        let drill_id = format!("ops.drill.{id_suffix}");
        if !declared_drills.insert(drill_id.clone()) {
            violations.push(violation(
                "OPS_DRILL_OWNERSHIP_DUPLICATE",
                format!("duplicate drill ownership entry for `{drill_id}`"),
                "keep exactly one ownership entry per drill id",
                Some(ownership_rel),
            ));
        }
        if !owner_values.contains(owner) {
            violations.push(violation(
                "OPS_DRILL_OWNERSHIP_OWNER_UNKNOWN",
                format!(
                    "drill ownership entry `{drill_id}` references unknown owner `{owner}` not present in ops/inventory/owners.json"
                ),
                "use an owner id defined in ops/inventory/owners.json areas values",
                Some(ownership_rel),
            ));
        }
    }

    let missing = drill_ids
        .difference(&declared_drills)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        violations.push(violation(
            "OPS_DRILL_OWNERSHIP_MISSING",
            format!(
                "missing ownership declarations for inventory drill ids: {}",
                missing.join(", ")
            ),
            "add ownership entries for every drill in ops/inventory/drills.json",
            Some(ownership_rel),
        ));
    }
    let stale = declared_drills
        .difference(&drill_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !stale.is_empty() {
        violations.push(violation(
            "OPS_DRILL_OWNERSHIP_STALE",
            format!(
                "ownership declarations exist for non-inventory drill ids: {}",
                stale.join(", ")
            ),
            "remove stale drill ownership entries or restore the drill ids in ops/inventory/drills.json",
            Some(ownership_rel),
        ));
    }

    Ok(())
}
