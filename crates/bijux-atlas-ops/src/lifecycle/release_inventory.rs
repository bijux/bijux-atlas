// SPDX-License-Identifier: Apache-2.0

use super::evidence_support::sha256_file;
use std::path::Path;

pub fn collect_observability_assets(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for rel in [
        "configs/schemas/contracts/observability/log.schema.json",
        "configs/schemas/contracts/observability/metrics.schema.json",
        "configs/sources/operations/observability/error-codes.json",
        "configs/sources/operations/observability/label-policy.json",
        "ops/observe/dashboards/atlas-observability-dashboard.json",
        "ops/observe/alerts/atlas-alert-rules.yaml",
        "ops/observe/slo-definitions.json",
        "ops/schema/k8s/obs-verify.schema.json",
        "ops/schema/observe/dashboard.schema.json",
        "ops/schema/observe/prometheus-rule.schema.json",
    ] {
        let path = repo_root.join(rel);
        if path.exists() {
            paths.push(rel.to_string());
        } else {
            return Err(format!("required observability asset missing: {rel}"));
        }
    }
    Ok(paths)
}

pub fn collect_perf_assets(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for rel in [
        "configs/sources/operations/perf/slo.yaml",
        "configs/sources/operations/perf/budgets.yaml",
        "configs/sources/operations/perf/benches.json",
        "configs/sources/operations/perf/exceptions.json",
        "configs/schemas/contracts/perf/slo.schema.json",
        "configs/schemas/contracts/perf/budgets.schema.json",
        "configs/schemas/contracts/perf/benches.schema.json",
        "configs/schemas/contracts/perf/load-report.schema.json",
        "configs/schemas/contracts/perf/exceptions.schema.json",
        "configs/schemas/contracts/perf/cold-start-report.schema.json",
        "ops/report/gene-lookup-baseline.json",
        "ops/schema/k8s/perf-on-kind.schema.json",
    ] {
        let path = repo_root.join(rel);
        if path.exists() {
            paths.push(rel.to_string());
        } else {
            return Err(format!("required perf asset missing: {rel}"));
        }
    }
    for rel in [
        "artifacts/perf/perf-slo.json",
        "artifacts/perf/gene-lookup-load.json",
        "artifacts/perf/cold-start.json",
        "artifacts/perf/perf-on-kind.json",
    ] {
        let path = repo_root.join(rel);
        if path.exists() {
            paths.push(rel.to_string());
        }
    }
    Ok(paths)
}

pub fn collect_dataset_assets(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for rel in [
        "configs/sources/runtime/datasets/manifest.yaml",
        "configs/sources/runtime/datasets/pinned-policy.yaml",
        "configs/schemas/contracts/datasets/manifest.schema.json",
        "configs/schemas/contracts/datasets/pinned-policy.schema.json",
        "configs/schemas/contracts/datasets/ingest-plan.schema.json",
        "configs/schemas/contracts/datasets/ingest-run.schema.json",
        "configs/schemas/contracts/datasets/endtoend.schema.json",
    ] {
        let path = repo_root.join(rel);
        if path.exists() {
            paths.push(rel.to_string());
        } else {
            return Err(format!("required dataset asset missing: {rel}"));
        }
    }
    for rel in [
        "artifacts/datasets/datasets-manifest.json",
        "artifacts/ingest/ingest-plan.json",
        "artifacts/ingest/ingest-run.json",
        "artifacts/ingest/endtoend-ingest-query.json",
    ] {
        let path = repo_root.join(rel);
        if path.exists() {
            paths.push(rel.to_string());
        }
    }
    Ok(paths)
}

pub fn collect_governance_assets(repo_root: &Path) -> Result<Vec<String>, String> {
    let paths = [
        "configs/sources/governance/governance/exceptions.yaml",
        "configs/sources/governance/governance/exceptions-archive.yaml",
        "configs/sources/governance/governance/compatibility.yaml",
        "configs/sources/governance/governance/deprecations.yaml",
        "configs/schemas/contracts/governance/exceptions.schema.json",
        "configs/schemas/contracts/governance/exceptions-archive.schema.json",
        "configs/schemas/contracts/governance/compatibility.schema.json",
        "configs/schemas/contracts/governance/deprecations.schema.json",
        "configs/schemas/contracts/reports/exceptions-summary.schema.json",
        "configs/schemas/contracts/reports/exceptions-expiry-warning.schema.json",
        "configs/schemas/contracts/reports/exceptions-churn.schema.json",
        "configs/schemas/contracts/reports/deprecations-summary.schema.json",
        "configs/schemas/contracts/reports/compat-warnings.schema.json",
        "configs/schemas/contracts/reports/breaking-changes.schema.json",
        "configs/schemas/contracts/reports/governance-doctor.schema.json",
        "configs/schemas/contracts/reports/institutional-delta-inputs.schema.json",
        "artifacts/governance/exceptions-summary.json",
        "artifacts/governance/exceptions-expiry-warning.json",
        "artifacts/governance/exceptions-churn.json",
        "artifacts/governance/exceptions-table.md",
        "artifacts/governance/deprecations-summary.json",
        "artifacts/governance/compat-warnings.json",
        "artifacts/governance/breaking-changes.json",
        "artifacts/governance/governance-doctor.json",
        "artifacts/governance/institutional-delta.md",
        "artifacts/governance/institutional-delta-inputs.json",
    ];
    let mut rows = Vec::new();
    for relative in paths {
        let path = repo_root.join(relative);
        if !path.exists() {
            return Err(format!("missing governance asset {}", path.display()));
        }
        rows.push(relative.to_string());
    }
    Ok(rows)
}

pub fn collect_report_paths(repo_root: &Path, run_id: &str) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for dir in [
        repo_root.join("ops/report/generated"),
        repo_root.join("artifacts/ops").join(run_id).join("reports"),
    ] {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&dir)
            .map_err(|err| format!("failed to read {}: {err}", dir.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let path = entry.path();
            if path.extension().and_then(|v| v.to_str()) != Some("json") {
                continue;
            }
            paths.push(
                path.strip_prefix(repo_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
            );
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub fn collect_simulation_summary_paths(repo_root: &Path, run_id: &str) -> Vec<String> {
    let reports_dir = repo_root.join("artifacts/ops").join(run_id).join("reports");
    ["ops-simulation-summary.json", "ops-lifecycle-summary.json"]
        .into_iter()
        .map(|name| reports_dir.join(name))
        .filter(|path| path.exists())
        .map(|path| {
            path.strip_prefix(repo_root)
                .unwrap_or(&path)
                .display()
                .to_string()
        })
        .collect::<Vec<_>>()
}

pub fn collect_drill_summary_paths(repo_root: &Path, run_id: &str) -> Vec<String> {
    let path = repo_root
        .join("artifacts/ops")
        .join(run_id)
        .join("reports")
        .join("ops-drills-summary.json");
    if path.exists() {
        vec![path
            .strip_prefix(repo_root)
            .unwrap_or(&path)
            .display()
            .to_string()]
    } else {
        Vec::new()
    }
}

pub fn collect_docs_site_summary(repo_root: &Path) -> Result<serde_json::Value, String> {
    let site_dir = repo_root.join("artifacts/docs/site");
    let mut file_count = 0usize;
    let mut stack = if site_dir.exists() {
        vec![site_dir.clone()]
    } else {
        Vec::new()
    };
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else {
                file_count += 1;
            }
        }
    }
    let index_path = site_dir.join("index.html");
    Ok(serde_json::json!({
        "site_dir": site_dir.strip_prefix(repo_root).unwrap_or(&site_dir).display().to_string(),
        "file_count": file_count,
        "sha256": if index_path.exists() {
            Some(sha256_file(&index_path)?)
        } else {
            None
        }
    }))
}

pub fn collect_supply_chain_inventory(repo_root: &Path) -> Result<Vec<serde_json::Value>, String> {
    let paths = [
        ".github/dependabot.yml",
        "configs/sources/repository/docs/package-lock.json",
        "configs/sources/repository/docs/requirements.lock.txt",
        "configs/sources/security/dependency-source-policy.json",
    ];
    let mut rows = Vec::new();
    for relative in paths {
        let path = repo_root.join(relative);
        if !path.exists() {
            return Err(format!(
                "missing supply-chain inventory file {}",
                path.display()
            ));
        }
        rows.push(serde_json::json!({
            "path": relative,
            "sha256": sha256_file(&path)?
        }));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::{
        collect_drill_summary_paths, collect_report_paths, collect_simulation_summary_paths,
    };
    use std::path::PathBuf;

    fn observe_sample_report_path(root: &std::path::Path, run_id: &str, name: &str) -> PathBuf {
        root.join("artifacts/ops")
            .join(run_id)
            .join("reports")
            .join(name)
    }

    #[test]
    fn report_paths_cover_generated_and_run_reports() {
        let root = tempfile::tempdir().expect("tempdir");
        let generated_dir = root.path().join("ops/report/generated");
        let reports_dir = root.path().join("artifacts/ops/run-local/reports");
        std::fs::create_dir_all(&generated_dir).expect("mkdir generated");
        std::fs::create_dir_all(&reports_dir).expect("mkdir reports");
        std::fs::write(generated_dir.join("summary.json"), "{}").expect("write generated report");
        std::fs::write(reports_dir.join("ops-lifecycle-summary.json"), "{}")
            .expect("write lifecycle report");

        let rows = collect_report_paths(root.path(), "run-local").expect("collect report paths");

        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn simulation_and_drill_summary_paths_only_return_existing_reports() {
        let root = tempfile::tempdir().expect("tempdir");
        let reports_dir = root.path().join("artifacts/ops/run-local/reports");
        std::fs::create_dir_all(&reports_dir).expect("mkdir reports");
        std::fs::write(
            observe_sample_report_path(root.path(), "run-local", "ops-simulation-summary.json"),
            "{}",
        )
        .expect("write simulation summary");
        std::fs::write(
            observe_sample_report_path(root.path(), "run-local", "ops-drills-summary.json"),
            "{}",
        )
        .expect("write drill summary");

        let simulation = collect_simulation_summary_paths(root.path(), "run-local");
        let drill = collect_drill_summary_paths(root.path(), "run-local");

        assert_eq!(simulation.len(), 1);
        assert_eq!(drill.len(), 1);
    }
}
