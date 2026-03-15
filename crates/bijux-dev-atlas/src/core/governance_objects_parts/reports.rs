use super::*;

fn read_required_contracts(repo_root: &Path) -> Vec<serde_json::Value> {
    read_json(&repo_root.join("ops/policy/required-contracts.json"))
        .ok()
        .and_then(|value| value.get("contracts").and_then(|v| v.as_array()).cloned())
        .unwrap_or_default()
}

fn read_lane_surface(repo_root: &Path) -> Vec<serde_json::Value> {
    read_json(&repo_root.join("configs/sources/repository/ci/lane-surface.json"))
        .ok()
        .and_then(|value| value.get("lanes").and_then(|v| v.as_array()).cloned())
        .unwrap_or_default()
}

fn read_check_report_map(repo_root: &Path) -> Vec<serde_json::Value> {
    read_json(&repo_root.join("configs/registry/reports/check-report-map.json"))
        .ok()
        .and_then(|value| value.get("mappings").and_then(|v| v.as_array()).cloned())
        .unwrap_or_default()
}

fn read_policy_step_registry(repo_root: &Path) -> Vec<serde_json::Value> {
    read_json(&repo_root.join("configs/sources/repository/ci/policy-outside-control-plane.json"))
        .ok()
        .and_then(|value| value.get("entries").and_then(|v| v.as_array()).cloned())
        .unwrap_or_default()
}

fn mapped_lane_name(raw: &str) -> &str {
    match raw {
        "pr" => "ci-pr-fast",
        "merge" => "ci-pr-full",
        "release" => "release-candidate",
        other => other,
    }
}

pub(super) fn governance_summary(objects: &[GovernanceObject]) -> BTreeMap<String, usize> {
    let mut by_domain = BTreeMap::<String, usize>::new();
    for obj in objects {
        *by_domain.entry(obj.domain.clone()).or_insert(0) += 1;
    }
    by_domain
}

pub(super) fn governance_summary_markdown(objects: &[GovernanceObject]) -> String {
    let by_domain = governance_summary(objects);
    let mut out = String::new();
    out.push_str("# Governance Summary\n\n");
    out.push_str("| Domain | Objects |\n| --- | --- |\n");
    for (domain, count) in by_domain {
        out.push_str(&format!("| `{domain}` | `{count}` |\n"));
    }
    out
}

pub(super) fn governance_summary_paths(repo_root: &Path) -> (PathBuf, PathBuf) {
    (
        repo_root.join("artifacts/governance/governance-graph.json"),
        repo_root.join("artifacts/governance/governance-summary.md"),
    )
}

pub(super) fn governance_index_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/governance-index.json")
}

pub(super) fn governance_contract_coverage_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/contract-coverage-map.json")
}

pub(super) fn governance_lane_coverage_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/lane-coverage-map.json")
}

pub(super) fn governance_orphan_checks_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/orphan-checks.json")
}

pub(super) fn governance_policy_surface_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/policy-surface-map.json")
}

pub(super) fn governance_drift_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/governance-drift.json")
}

pub(super) fn governance_version(
    objects: &[GovernanceObject],
    contracts: &[serde_json::Value],
    lanes: &[serde_json::Value],
    report_mappings: &[serde_json::Value],
) -> String {
    let payload = serde_json::json!({
        "objects": objects,
        "contracts": contracts,
        "lanes": lanes,
        "report_mappings": report_mappings,
    });
    let encoded = serde_json::to_vec(&payload).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    format!("{:x}", hasher.finalize())
}

pub(super) fn governance_index_payload(
    repo_root: &Path,
    objects: &[GovernanceObject],
) -> serde_json::Value {
    let contracts = read_required_contracts(repo_root);
    let lanes = read_lane_surface(repo_root);
    let report_mappings = read_check_report_map(repo_root);
    let version = governance_version(objects, &contracts, &lanes, &report_mappings);
    let mut rows = Vec::new();
    for contract in contracts {
        let Some(contract_id) = contract.get("contract_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let domain = contract
            .get("domain")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let lanes_for_contract = contract
            .get("lanes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect::<Vec<_>>();
        let reports = report_mappings
            .iter()
            .filter(|row| row.get("check_id").and_then(|v| v.as_str()) == Some(contract_id))
            .map(|row| {
                serde_json::json!({
                    "report_id": row.get("report_id").cloned().unwrap_or(serde_json::Value::Null),
                    "report_path": row.get("report_path").cloned().unwrap_or(serde_json::Value::Null),
                    "evidence_level": row.get("evidence_level").cloned().unwrap_or(serde_json::Value::Null),
                })
            })
            .collect::<Vec<_>>();
        rows.push(serde_json::json!({
            "domain": domain,
            "contract_id": contract_id,
            "owner": contract.get("owner").cloned().unwrap_or(serde_json::Value::Null),
            "lanes": lanes_for_contract,
            "reports": reports,
        }));
    }
    rows.sort_by(|left, right| {
        left.get("domain")
            .and_then(|v| v.as_str())
            .cmp(&right.get("domain").and_then(|v| v.as_str()))
            .then_with(|| {
                left.get("contract_id")
                    .and_then(|v| v.as_str())
                    .cmp(&right.get("contract_id").and_then(|v| v.as_str()))
            })
    });
    serde_json::json!({
        "schema_version": 1,
        "report_id": "governance-index",
        "version": 1,
        "kind": "governance_index",
        "governance_version": version,
        "inputs": {
            "domain_registry_map": "ops/governance/repository/domain-registry-map.json",
            "required_contracts": "ops/policy/required-contracts.json",
            "lane_surface": "configs/sources/repository/ci/lane-surface.json",
            "check_report_map": "configs/registry/reports/check-report-map.json"
        },
        "domains": governance_summary(objects),
        "contracts": rows,
        "summary": {
            "domain_count": governance_summary(objects).len(),
            "contract_count": rows.len()
        },
        "evidence": {
            "authority_objects": objects.len(),
            "artifacts_root": "artifacts/governance"
        }
    })
}

pub(super) fn governance_contract_coverage_payload(repo_root: &Path) -> serde_json::Value {
    let contracts = read_required_contracts(repo_root);
    let rows = contracts
        .into_iter()
        .map(|contract| {
            let lanes = contract
                .get("lanes")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "contract_id": contract.get("contract_id").cloned().unwrap_or(serde_json::Value::Null),
                "domain": contract.get("domain").cloned().unwrap_or(serde_json::Value::Null),
                "lanes": lanes,
                "covered": !lanes.is_empty(),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema_version": 1,
        "report_id": "contract-coverage-map",
        "version": 1,
        "kind": "contract_coverage_map",
        "rows": rows,
        "inputs": {
            "required_contracts": "ops/policy/required-contracts.json"
        },
        "summary": {
            "total": rows.len(),
            "covered": rows.iter().filter(|row| row.get("covered").and_then(|v| v.as_bool()) == Some(true)).count()
        },
        "evidence": {
            "lane_source": "configs/sources/repository/ci/lane-surface.json"
        }
    })
}

pub(super) fn governance_lane_coverage_payload(repo_root: &Path) -> serde_json::Value {
    let lanes = read_lane_surface(repo_root);
    let contracts = read_required_contracts(repo_root);
    let mut rows = Vec::new();
    for lane in lanes {
        let Some(lane_name) = lane.get("lane").and_then(|v| v.as_str()) else {
            continue;
        };
        let covered_contracts = contracts
            .iter()
            .filter(|contract| {
                contract
                    .get("lanes")
                    .and_then(|v| v.as_array())
                    .is_some_and(|lanes| {
                        lanes.iter().any(|value| {
                            value.as_str() == Some(lane_name)
                                || mapped_lane_name(value.as_str().unwrap_or_default()) == lane_name
                        })
                    })
            })
            .filter_map(|contract| {
                contract
                    .get("contract_id")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            })
            .collect::<Vec<_>>();
        rows.push(serde_json::json!({
            "lane": lane_name,
            "workflow": lane.get("workflow").cloned().unwrap_or(serde_json::Value::Null),
            "contracts": covered_contracts,
        }));
    }
    serde_json::json!({
        "schema_version": 1,
        "report_id": "lane-coverage-map",
        "version": 1,
        "kind": "lane_coverage_map",
        "rows": rows,
        "inputs": {
            "lane_surface": "configs/sources/repository/ci/lane-surface.json",
            "required_contracts": "ops/policy/required-contracts.json"
        },
        "summary": {
            "total": rows.len()
        },
        "evidence": {
            "workflow_registry": "configs/sources/repository/ci/lane-surface.json"
        }
    })
}

pub(super) fn governance_orphan_checks_payload(repo_root: &Path) -> serde_json::Value {
    let lanes = read_lane_surface(repo_root);
    let contracts = read_required_contracts(repo_root);
    let mapped_contracts = contracts
        .iter()
        .filter_map(|row| row.get("contract_id").and_then(|v| v.as_str()))
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for lane in lanes {
        for command in lane
            .get("commands")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
        {
            let id = command
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if id.contains('-')
                && id
                    .chars()
                    .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '-')
                && !mapped_contracts.contains(id)
            {
                rows.push(serde_json::json!({
                    "lane": lane.get("lane").cloned().unwrap_or(serde_json::Value::Null),
                    "check_id": id,
                    "reason": "lane references uppercase contract-like id without required-contract mapping",
                }));
            }
        }
    }
    serde_json::json!({
        "schema_version": 1,
        "report_id": "orphan-checks",
        "version": 1,
        "kind": "orphan_checks",
        "rows": rows,
        "inputs": {
            "lane_surface": "configs/sources/repository/ci/lane-surface.json",
            "required_contracts": "ops/policy/required-contracts.json"
        },
        "summary": {
            "total": rows.len()
        },
        "evidence": {
            "coverage_model": "lane-to-required-contract cross-check"
        }
    })
}

pub(super) fn governance_policy_surface_payload(repo_root: &Path) -> serde_json::Value {
    let mut rows = read_policy_step_registry(repo_root)
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "workflow": entry.get("workflow").cloned().unwrap_or(serde_json::Value::Null),
                "step": entry.get("step").cloned().unwrap_or(serde_json::Value::Null),
                "classification": entry.get("classification").cloned().unwrap_or(serde_json::Value::Null),
                "owner": entry.get("owner").cloned().unwrap_or(serde_json::Value::Null),
                "replacement": entry.get("control_plane_command").cloned().unwrap_or(serde_json::Value::Null),
                "source": "configs/sources/repository/ci/policy-outside-control-plane.json",
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.get("workflow")
            .and_then(|v| v.as_str())
            .cmp(&right.get("workflow").and_then(|v| v.as_str()))
            .then_with(|| {
                left.get("step")
                    .and_then(|v| v.as_str())
                    .cmp(&right.get("step").and_then(|v| v.as_str()))
            })
    });
    serde_json::json!({
        "schema_version": 1,
        "report_id": "policy-surface-map",
        "version": 1,
        "kind": "policy_surface_map",
        "rows": rows,
        "inputs": {
            "policy_registry": "configs/sources/repository/ci/policy-outside-control-plane.json"
        },
        "summary": {
            "total": rows.len()
        },
        "evidence": {
            "replacement_source": "configs/sources/repository/ci/policy-outside-control-plane.json"
        }
    })
}

pub(super) fn governance_drift_payload(
    current_index: &serde_json::Value,
    previous_index: Option<&serde_json::Value>,
) -> serde_json::Value {
    let current_contracts = current_index
        .get("contracts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let previous_contracts = previous_index
        .and_then(|value| value.get("contracts"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let current_ids = current_contracts
        .iter()
        .filter_map(|row| {
            row.get("contract_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    let previous_ids = previous_contracts
        .iter()
        .filter_map(|row| {
            row.get("contract_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    let added = current_ids
        .difference(&previous_ids)
        .cloned()
        .collect::<Vec<_>>();
    let removed = previous_ids
        .difference(&current_ids)
        .cloned()
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema_version": 1,
        "report_id": "governance-drift",
        "version": 1,
        "kind": "governance_drift",
        "added_contracts": added,
        "removed_contracts": removed,
        "changed": !added.is_empty() || !removed.is_empty(),
        "inputs": {
            "baseline": "artifacts/governance/governance-index.json",
            "candidate": "artifacts/governance/governance-index.json"
        },
        "summary": {
            "added": added.len(),
            "removed": removed.len()
        },
        "evidence": {
            "comparison_scope": "contract ids"
        }
    })
}

pub(super) fn governance_coverage_score(objects: &[GovernanceObject]) -> serde_json::Value {
    let total = objects.len() as f64;
    let mut proof_ready = 0usize;
    for object in objects {
        let complete = !object.owner.is_empty()
            && !object.reviewed_on.is_empty()
            && !object.consumers.is_empty()
            && !object.evidence.is_empty()
            && !object.links.is_empty()
            && !object.authority_source.is_empty();
        if complete {
            proof_ready += 1;
        }
    }
    let score = if total == 0.0 {
        0.0
    } else {
        ((proof_ready as f64 / total) * 10000.0).round() / 100.0
    };
    serde_json::json!({
        "schema_version": 1,
        "kind": "governance_coverage_score",
        "total_objects": objects.len(),
        "proof_ready_objects": proof_ready,
        "coverage_percent": score
    })
}

pub(super) fn governance_coverage_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/governance-coverage.json")
}

pub(super) fn governance_orphan_report_path(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts/governance/governance-orphans.json")
}

pub(super) fn governance_orphan_report_payload(rows: &[serde_json::Value]) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "kind": "governance_orphans",
        "orphans": rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_drift_payload_is_stable_for_identical_index() {
        let current = serde_json::json!({
            "contracts": [
                {"contract_id": "DOC-001"},
                {"contract_id": "OPS-001"}
            ]
        });
        let drift = governance_drift_payload(&current, Some(&current));
        assert_eq!(drift["changed"].as_bool(), Some(false));
        assert_eq!(
            drift["added_contracts"].as_array().map(|rows| rows.len()),
            Some(0)
        );
        assert_eq!(
            drift["removed_contracts"].as_array().map(|rows| rows.len()),
            Some(0)
        );
    }

    #[test]
    fn governance_version_changes_when_contracts_change() {
        let objects = vec![GovernanceObject {
            id: "docs:page:index".to_string(),
            domain: "docs".to_string(),
            owner: "docs".to_string(),
            consumers: vec!["docs/index.md".to_string()],
            lifecycle: "stable".to_string(),
            evidence: vec!["artifacts/governance/docs/pages.json".to_string()],
            links: vec!["docs/index.md".to_string()],
            authority_source: "docs/".to_string(),
            reviewed_on: "2026-03-03".to_string(),
        }];
        let lanes = vec![serde_json::json!({"lane": "ci-pr-fast"})];
        let reports = vec![serde_json::json!({"check_id": "DOC-001"})];
        let version_a = governance_version(
            &objects,
            &[serde_json::json!({"contract_id": "DOC-001"})],
            &lanes,
            &reports,
        );
        let version_b = governance_version(
            &objects,
            &[serde_json::json!({"contract_id": "DOC-002"})],
            &lanes,
            &reports,
        );
        assert_ne!(version_a, version_b);
    }
}
