// SPDX-License-Identifier: Apache-2.0

pub(super) fn checks_ops_human_workflow_maturity(
    ctx: &CheckContext<'_>,
) -> Result<Vec<Violation>, CheckError> {
    let mut violations = Vec::new();
    validate_root_ops_docs(ctx, &mut violations)?;
    validate_drill_cross_links(ctx, &mut violations)?;
    validate_generated_workflow_reports(ctx, &mut violations)?;
    Ok(violations)
}

fn validate_root_ops_docs(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let requirements = [
        (
            "ops/README.md",
            [
                "Machine validation entrypoint:",
                "Human walkthroughs and procedures live in `docs/",
                "Root Docs",
            ]
            .as_slice(),
        ),
        (
            "ops/CONTRACT.md",
            [
                "## Scope",
                "## Durable Rules",
                "## Machine Authorities",
                "## Evidence",
                "## Minimal Release Surface",
            ]
            .as_slice(),
        ),
        (
            "ops/INDEX.md",
            [
                "Canonical ops pillars:",
                "Generated directories",
                "Schema registry:",
            ]
            .as_slice(),
        ),
        (
            "ops/SSOT.md",
            [
                "## Allowed Root Markdown",
                "## Forbidden Markdown Shape",
                "## Rationale",
            ]
            .as_slice(),
        ),
        (
            "ops/ERRORS.md",
            [
                "Use `bijux-dev-atlas ops validate --format json`",
                "REPO-LAW-001",
                "REPO-LAW-004",
            ]
            .as_slice(),
        ),
    ];

    for (rel_str, snippets) in requirements {
        let rel = Path::new(rel_str);
        let text = fs::read_to_string(ctx.repo_root.join(rel))
            .map_err(|err| CheckError::Failed(format!("read {}: {err}", rel.display())))?;
        for snippet in snippets {
            if !text.contains(snippet) {
                violations.push(violation(
                    "OPS_ROOT_DOC_INCOMPLETE",
                    format!("ops root doc `{}` is missing `{snippet}`", rel.display()),
                    "keep the five root docs complete and aligned with the live ops layout",
                    Some(rel),
                ));
            }
        }
    }

    Ok(())
}

fn validate_drill_cross_links(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let drills_rel = Path::new("ops/inventory/drills.json");
    let drills_text = fs::read_to_string(ctx.repo_root.join(drills_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", drills_rel.display())))?;
    let drills_json: serde_json::Value = serde_json::from_str(&drills_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", drills_rel.display())))?;
    let drill_ids = drills_json
        .get("drills")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let links_rel = Path::new("ops/inventory/drill-contract-links.json");
    let links_text = fs::read_to_string(ctx.repo_root.join(links_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", links_rel.display())))?;
    let links_json: serde_json::Value = serde_json::from_str(&links_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", links_rel.display())))?;
    let linked_ids = links_json
        .get("links")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("drill_id").and_then(|value| value.as_str()))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let missing = drill_ids
        .difference(&linked_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        violations.push(violation(
            "OPS_DRILL_CONTRACT_LINKAGE_MISSING",
            format!(
                "inventory drills are missing contract linkage entries: {}",
                missing.join(", ")
            ),
            "link every drill id from ops/inventory/drills.json in ops/inventory/drill-contract-links.json",
            Some(links_rel),
        ));
    }

    Ok(())
}

fn validate_generated_workflow_reports(
    ctx: &CheckContext<'_>,
    violations: &mut Vec<Violation>,
) -> Result<(), CheckError> {
    let incident_rel = Path::new("ops/_generated.example/incident-playbook-generation-report.json");
    let incident_text = fs::read_to_string(ctx.repo_root.join(incident_rel))
        .map_err(|err| CheckError::Failed(format!("read {}: {err}", incident_rel.display())))?;
    let incident_json: serde_json::Value = serde_json::from_str(&incident_text)
        .map_err(|err| CheckError::Failed(format!("parse {}: {err}", incident_rel.display())))?;
    if incident_json.get("status").and_then(|value| value.as_str()) != Some("pass") {
        violations.push(violation(
            "OPS_INCIDENT_PLAYBOOK_REPORT_BLOCKING",
            "incident playbook generation report status is not `pass`".to_string(),
            "refresh ops/_generated.example/incident-playbook-generation-report.json after fixing playbook generation drift",
            Some(incident_rel),
        ));
    }

    let playbooks = incident_json
        .get("playbooks")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    if playbooks.is_empty() {
        violations.push(violation(
            "OPS_INCIDENT_PLAYBOOK_REPORT_EMPTY",
            "incident playbook generation report must include playbooks entries".to_string(),
            "include representative playbooks in ops/_generated.example/incident-playbook-generation-report.json",
            Some(incident_rel),
        ));
    }

    Ok(())
}
