// SPDX-License-Identifier: Apache-2.0
//! Shared evidence helpers for install-status flows.

use super::*;

pub(super) fn build_lifecycle_evidence_bundle(
    repo_root: &std::path::Path,
    run_id: &RunId,
) -> Result<serde_json::Value, String> {
    let run_root = repo_root.join("artifacts/ops").join(run_id.as_str());
    let evidence_dir = run_root.join("evidence");
    std::fs::create_dir_all(&evidence_dir)
        .map_err(|err| format!("failed to create {}: {err}", evidence_dir.display()))?;
    let list_path = evidence_dir.join("ops-lifecycle-evidence.list");
    let tar_path = evidence_dir.join("ops-lifecycle-evidence.tar");
    let mut files = Vec::<String>::new();
    for dir in [run_root.join("reports"), run_root.join("debug")] {
        if !dir.exists() {
            continue;
        }
        let mut stack = vec![dir];
        while let Some(path) = stack.pop() {
            let entries = std::fs::read_dir(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            for entry in entries {
                let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path);
                    continue;
                }
                let rel = entry_path
                    .strip_prefix(repo_root)
                    .map_err(|err| format!("failed to relativize {}: {err}", entry_path.display()))?
                    .display()
                    .to_string();
                files.push(rel);
            }
        }
    }
    files.sort();
    files.dedup();
    std::fs::write(&list_path, files.join("\n"))
        .map_err(|err| format!("failed to write {}: {err}", list_path.display()))?;
    if files.is_empty() {
        return Ok(serde_json::json!({
            "status": "skipped",
            "tar_path": tar_path.display().to_string(),
            "list_path": list_path.display().to_string(),
            "files": files
        }));
    }
    let output = std::process::Command::new("tar")
        .current_dir(repo_root)
        .args([
            "--sort=name",
            "-cf",
            &tar_path.display().to_string(),
            "-T",
            &list_path.display().to_string(),
        ])
        .output()
        .map_err(|err| format!("failed to execute tar for lifecycle evidence bundle: {err}"))?;
    let status = if output.status.success() { "ok" } else { "failed" };
    Ok(serde_json::json!({
        "status": status,
        "tar_path": tar_path.display().to_string(),
        "list_path": list_path.display().to_string(),
        "files": files,
        "stdout": String::from_utf8_lossy(&output.stdout).trim().to_string(),
        "stderr": String::from_utf8_lossy(&output.stderr).trim().to_string()
    }))
}

pub(super) fn evidence_root(repo_root: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let path = repo_root.join("release/evidence");
    std::fs::create_dir_all(&path)
        .map_err(|err| format!("failed to create {}: {err}", path.display()))?;
    Ok(path)
}

pub(super) fn sha256_file(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    use sha2::{Digest, Sha256};
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub(super) fn package_chart_for_evidence(
    process: &OpsProcess,
    repo_root: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let evidence_root = evidence_root(repo_root)?;
    let package_dir = evidence_root.join("packages");
    std::fs::create_dir_all(&package_dir)
        .map_err(|err| format!("failed to create {}: {err}", package_dir.display()))?;
    let chart_path = simulation_current_chart_path(repo_root);
    let argv = vec![
        "package".to_string(),
        chart_path.display().to_string(),
        "--destination".to_string(),
        package_dir.display().to_string(),
    ];
    process
        .run_subprocess("helm", &argv, repo_root)
        .map_err(|err| err.to_stable_message())?;
    let mut packages = std::fs::read_dir(&package_dir)
        .map_err(|err| format!("failed to read {}: {err}", package_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("tgz"))
        .collect::<Vec<_>>();
    packages.sort();
    packages
        .pop()
        .ok_or_else(|| format!("no chart package produced in {}", package_dir.display()))
}

pub(super) fn collect_image_artifacts(repo_root: &std::path::Path) -> Result<Vec<serde_json::Value>, String> {
    let values_root = repo_root.join("ops/k8s/values");
    let mut rows = std::fs::read_dir(&values_root)
        .map_err(|err| format!("failed to read {}: {err}", values_root.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("yaml"))
        .collect::<Vec<_>>();
    rows.sort();
    let mut artifacts = Vec::new();
    for path in rows {
        let value: serde_yaml::Value = serde_yaml::from_str(
            &std::fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        )
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
        let Some(image) = value.get("image") else {
            continue;
        };
        let repository = image
            .get("repository")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let digest = image
            .get("digest")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let tag = image
            .get("tag")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if repository.is_empty() && digest.is_empty() && tag.is_empty() {
            continue;
        }
        let profile = path
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
        artifacts.push(serde_json::json!({
            "source_path": path.strip_prefix(repo_root).unwrap_or(&path).display().to_string(),
            "profile": profile,
            "repository": repository,
            "digest": digest,
            "tag": tag
        }));
    }
    Ok(artifacts)
}

pub(super) fn reset_directory(path: &std::path::Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_dir_all(path)
            .map_err(|err| format!("failed to clear {}: {err}", path.display()))?;
    }
    std::fs::create_dir_all(path).map_err(|err| format!("failed to create {}: {err}", path.display()))
}

pub(super) fn image_ref_for_evidence(row: &serde_json::Value) -> Option<String> {
    let repository = row
        .get("repository")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim();
    let digest = row
        .get("digest")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim();
    if repository.is_empty() || digest.is_empty() {
        None
    } else {
        Some(format!("{repository}@{digest}"))
    }
}

pub(super) fn collect_sboms(
    repo_root: &std::path::Path,
    image_artifacts: &[serde_json::Value],
) -> Result<Vec<serde_json::Value>, String> {
    let evidence_root = evidence_root(repo_root)?;
    let sbom_dir = evidence_root.join("sboms");
    reset_directory(&sbom_dir)?;
    let mut rows = Vec::new();
    for image in image_artifacts {
        let profile = image
            .get("profile")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let digest = image
            .get("digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if digest.is_empty() {
            continue;
        }
        let image_ref = image_ref_for_evidence(image)
            .or_else(|| Some(digest.to_string()))
            .unwrap_or_else(|| digest.to_string());
        let sbom_path = sbom_dir.join(format!("{profile}.spdx.json"));
        let document = serde_json::json!({
            "SPDXID": "SPDXRef-DOCUMENT",
            "creationInfo": {
                "created": "1970-01-01T00:00:00Z",
                "creators": ["Tool: bijux-dev-atlas release evidence"],
                "licenseListVersion": "3.22"
            },
            "dataLicense": "CC0-1.0",
            "documentNamespace": format!("https://bijux.dev/evidence/sbom/{profile}/{digest}"),
            "name": format!("bijux-atlas {profile} image evidence"),
            "packages": [{
                "SPDXID": format!("SPDXRef-Package-{profile}"),
                "downloadLocation": "NOASSERTION",
                "externalRefs": [{
                    "referenceCategory": "PACKAGE-MANAGER",
                    "referenceLocator": image_ref,
                    "referenceType": "purl"
                }],
                "filesAnalyzed": false,
                "name": format!("bijux-atlas-{profile}"),
                "primaryPackagePurpose": "CONTAINER",
                "versionInfo": digest
            }],
            "relationships": [],
            "spdxVersion": "SPDX-2.3"
        });
        std::fs::write(
            &sbom_path,
            serde_json::to_string_pretty(&document).map_err(|err| err.to_string())?,
        )
        .map_err(|err| format!("failed to write {}: {err}", sbom_path.display()))?;
        rows.push(serde_json::json!({
            "path": sbom_path.strip_prefix(repo_root).unwrap_or(&sbom_path).display().to_string(),
            "format": "spdx-json",
            "sha256": sha256_file(&sbom_path)?,
            "image_ref": image_ref
        }));
    }
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(rows)
}

pub(super) fn collect_scan_reports(repo_root: &std::path::Path) -> Result<Vec<serde_json::Value>, String> {
    let scan_dir = evidence_root(repo_root)?.join("scans");
    if !scan_dir.exists() {
        return Ok(Vec::new());
    }
    let mut rows = std::fs::read_dir(&scan_dir)
        .map_err(|err| format!("failed to read {}: {err}", scan_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            let format = if name.ends_with(".json") {
                Some("json")
            } else if name.ends_with(".sarif") || name.ends_with(".sarif.json") {
                Some("sarif")
            } else {
                None
            }?;
            Some((path, format.to_string()))
        })
        .map(|(path, format)| {
            Ok(serde_json::json!({
                "path": path.strip_prefix(repo_root).unwrap_or(&path).display().to_string(),
                "format": format,
                "sha256": sha256_file(&path)?
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(rows)
}

pub(super) fn redact_sensitive_text(text: &str) -> String {
    let mut lines = Vec::new();
    for line in text.lines() {
        let upper = line.to_ascii_uppercase();
        if let Some((prefix, _)) = line.split_once('=') {
            let normalized = prefix.trim().to_ascii_uppercase();
            if ["PASSWORD", "TOKEN", "SECRET", "API_KEY"].contains(&normalized.as_str()) {
                lines.push(format!("{prefix}=[REDACTED]"));
                continue;
            }
        }
        if upper.contains("AUTHORIZATION: BEARER ") {
            lines.push("Authorization: Bearer [REDACTED]".to_string());
        } else {
            lines.push(line.to_string());
        }
    }
    if text.ends_with('\n') {
        format!("{}\n", lines.join("\n"))
    } else {
        lines.join("\n")
    }
}

pub(super) fn contains_common_secret_pattern(text: &str) -> bool {
    for line in text.lines() {
        let upper = line.to_ascii_uppercase();
        if let Some((prefix, value)) = line.split_once('=') {
            let normalized = prefix.trim().to_ascii_uppercase();
            if ["PASSWORD", "TOKEN", "SECRET", "API_KEY"].contains(&normalized.as_str())
                && value.trim() != "[REDACTED]"
            {
                return true;
            }
        }
        if upper.contains("AUTHORIZATION: BEARER ") && !upper.contains("AUTHORIZATION: BEARER [REDACTED]") {
            return true;
        }
    }
    false
}

pub(super) fn collect_redacted_logs(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let source_root = repo_root.join("artifacts/ops");
    let redacted_root = evidence_root(repo_root)?.join("redacted-logs");
    reset_directory(&redacted_root)?;
    if !source_root.exists() {
        return Ok(Vec::new());
    }
    let mut stack = vec![source_root];
    let mut outputs = Vec::new();
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            let relative = entry_path
                .strip_prefix(repo_root)
                .unwrap_or(&entry_path)
                .display()
                .to_string();
            if !relative.contains("/debug/") {
                continue;
            }
            let output_name = relative.replace('/', "__");
            let output_path = redacted_root.join(output_name);
            let source = std::fs::read_to_string(&entry_path)
                .unwrap_or_else(|_| String::from_utf8_lossy(&std::fs::read(&entry_path).unwrap_or_default()).to_string());
            let redacted = redact_sensitive_text(&source);
            std::fs::write(&output_path, redacted)
                .map_err(|err| format!("failed to write {}: {err}", output_path.display()))?;
            outputs.push(
                output_path
                    .strip_prefix(repo_root)
                    .unwrap_or(&output_path)
                    .display()
                    .to_string(),
            );
        }
    }
    outputs.sort();
    Ok(outputs)
}

pub(super) fn render_evidence_index_html(
    repo_root: &std::path::Path,
    manifest: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let index_path = evidence_root(repo_root)?.join("index.html");
    let html = format!(
        "<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>Release Evidence</title></head>\n<body>\n<h1>Release Evidence</h1>\n<p>Generated by bijux dev atlas ops evidence collect.</p>\n<ul>\n<li>Manifest: {}</li>\n<li>Identity: {}</li>\n<li>Chart package: {}</li>\n<li>SBOM count: {}</li>\n<li>Scan report count: {}</li>\n<li>Redacted logs: {}</li>\n</ul>\n</body>\n</html>\n",
        "release/evidence/manifest.json",
        manifest
            .get("identity_path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("release/evidence/identity.json"),
        manifest
            .get("chart_package")
            .and_then(|value| value.get("path"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("release/evidence/packages"),
        manifest
            .get("sboms")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len())
            .unwrap_or(0),
        manifest
            .get("scan_reports")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len())
            .unwrap_or(0),
        manifest
            .get("redacted_logs")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len())
            .unwrap_or(0),
    );
    std::fs::write(&index_path, html)
        .map_err(|err| format!("failed to write {}: {err}", index_path.display()))?;
    Ok(serde_json::json!({
        "path": index_path.strip_prefix(repo_root).unwrap_or(&index_path).display().to_string(),
        "sha256": sha256_file(&index_path)?
    }))
}

pub(super) fn collect_observability_assets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for rel in [
        "configs/contracts/observability/log.schema.json",
        "configs/contracts/observability/metrics.schema.json",
        "configs/contracts/observability/error-codes.json",
        "configs/contracts/observability/label-policy.json",
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

pub(super) fn collect_perf_assets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for rel in [
        "configs/perf/slo.yaml",
        "configs/perf/budgets.yaml",
        "configs/perf/benches.json",
        "configs/perf/exceptions.json",
        "configs/contracts/perf/slo.schema.json",
        "configs/contracts/perf/budgets.schema.json",
        "configs/contracts/perf/benches.schema.json",
        "configs/contracts/perf/load-report.schema.json",
        "configs/contracts/perf/exceptions.schema.json",
        "configs/contracts/perf/cold-start-report.schema.json",
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

pub(super) fn collect_dataset_assets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for rel in [
        "configs/datasets/manifest.yaml",
        "configs/datasets/pinned-policy.yaml",
        "configs/contracts/datasets/manifest.schema.json",
        "configs/contracts/datasets/pinned-policy.schema.json",
        "configs/contracts/datasets/ingest-plan.schema.json",
        "configs/contracts/datasets/ingest-run.schema.json",
        "configs/contracts/datasets/endtoend.schema.json",
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

pub(super) fn load_required_metric_names(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let contract_path = repo_root.join("configs/contracts/observability/metrics.schema.json");
    let contract: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&contract_path)
            .map_err(|err| format!("failed to read {}: {err}", contract_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", contract_path.display()))?;
    let mut rows = contract
        .get("required_metrics")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    rows.sort();
    rows.dedup();
    Ok(rows)
}

pub(super) fn collect_governance_assets(repo_root: &std::path::Path) -> Result<Vec<String>, String> {
    let paths = [
        "configs/governance/exceptions.yaml",
        "configs/governance/exceptions-archive.yaml",
        "configs/governance/compatibility.yaml",
        "configs/governance/deprecations.yaml",
        "configs/contracts/governance/exceptions.schema.json",
        "configs/contracts/governance/exceptions-archive.schema.json",
        "configs/contracts/governance/compatibility.schema.json",
        "configs/contracts/governance/deprecations.schema.json",
        "configs/contracts/reports/exceptions-summary.schema.json",
        "configs/contracts/reports/exceptions-expiry-warning.schema.json",
        "configs/contracts/reports/exceptions-churn.schema.json",
        "configs/contracts/reports/deprecations-summary.schema.json",
        "configs/contracts/reports/compat-warnings.schema.json",
        "configs/contracts/reports/breaking-changes.schema.json",
        "configs/contracts/reports/governance-doctor.schema.json",
        "configs/contracts/reports/institutional-delta-inputs.schema.json",
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

pub(super) fn observability_contract_checks(
    repo_root: &std::path::Path,
    metrics_body: &str,
) -> Result<serde_json::Value, String> {
    let required_metric_names = load_required_metric_names(repo_root)?;
    let required_metrics_present = required_metric_names
        .iter()
        .filter(|name| metrics_body.contains(&format!("{}{{", name)))
        .cloned()
        .collect::<Vec<_>>();
    let missing_metrics = required_metric_names
        .iter()
        .filter(|name| !metrics_body.contains(&format!("{}{{", name)))
        .cloned()
        .collect::<Vec<_>>();
    let warmup_lock_metrics_present = [
        "bijux_warmup_lock_contention_total{",
        "bijux_warmup_lock_expired_total{",
        "bijux_warmup_lock_wait_p95_seconds{",
    ]
    .iter()
    .all(|needle| metrics_body.contains(needle));

    let response_contract = std::fs::read_to_string(
        repo_root.join("crates/bijux-atlas-server/src/http/response_contract.rs"),
    )
    .map_err(|err| format!("failed to read response contract source: {err}"))?;
    let error_registry = std::fs::read_to_string(
        repo_root.join("configs/contracts/observability/error-codes.json"),
    )
    .map_err(|err| format!("failed to read error registry: {err}"))?;
    let openapi = std::fs::read_to_string(repo_root.join("crates/bijux-atlas-api/openapi.v1.json"))
        .map_err(|err| format!("failed to read openapi: {err}"))?;
    let error_registry_aligned = error_registry.contains("NotReady")
        && error_registry.contains("RateLimited")
        && response_contract.contains("ApiErrorCode::NotReady")
        && response_contract.contains("ApiErrorCode::RateLimited")
        && openapi.contains("\"ApiErrorCode\"");

    let main_rs = std::fs::read_to_string(repo_root.join("crates/bijux-atlas-server/src/main.rs"))
        .map_err(|err| format!("failed to read main.rs: {err}"))?;
    let startup_log_fields_present = main_rs.contains("event_id = \"startup\"")
        && main_rs.contains("release_id = %release_id")
        && main_rs.contains("governance_version = %governance_version");

    let redacted = redact_sensitive_text("TOKEN=secret-value\nAuthorization: Bearer abc123\n");
    let redaction_contract_passed =
        !contains_common_secret_pattern(&redacted) && !redacted.contains("secret-value") && !redacted.contains("abc123");

    let dashboard_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/observe/dashboard.schema.json"))
            .map_err(|err| format!("failed to read dashboard schema: {err}"))?,
    )
    .map_err(|err| format!("failed to parse dashboard schema: {err}"))?;
    let dashboard_contract: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/observe/contracts/dashboard-panels-contract.json"))
            .map_err(|err| format!("failed to read dashboard contract: {err}"))?,
    )
    .map_err(|err| format!("failed to parse dashboard contract: {err}"))?;
    let dashboard_text = std::fs::read_to_string(
        repo_root.join("ops/observe/dashboards/atlas-observability-dashboard.json"),
    )
    .map_err(|err| format!("failed to read dashboard: {err}"))?;
    let dashboard: serde_json::Value =
        serde_json::from_str(&dashboard_text).map_err(|err| format!("failed to parse dashboard: {err}"))?;
    let panel_titles = dashboard
        .get("panels")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("title").and_then(serde_json::Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let required_panels = dashboard_contract
        .get("required_panels")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let required_rows = dashboard_contract
        .get("required_rows")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let dashboard_contract_valid = dashboard_schema.get("type") == Some(&serde_json::Value::String("object".to_string()))
        && dashboard.get("uid").and_then(serde_json::Value::as_str).is_some()
        && dashboard.get("schemaVersion").and_then(serde_json::Value::as_i64).is_some()
        && !required_panels.iter().any(|name| !panel_titles.contains(name))
        && !required_rows.iter().any(|name| !panel_titles.contains(name));

    let slo_path = repo_root.join("ops/observe/slo-definitions.json");
    let slo_schema_path = repo_root.join("ops/schema/observe/slo-definitions.schema.json");
    let slo: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&slo_path)
            .map_err(|err| format!("failed to read {}: {err}", slo_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", slo_path.display()))?;
    let slo_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&slo_schema_path)
            .map_err(|err| format!("failed to read {}: {err}", slo_schema_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", slo_schema_path.display()))?;
    let slo_contract_valid = slo["schema_version"] == slo_schema["properties"]["schema_version"]["const"]
        && slo.get("slos").and_then(serde_json::Value::as_array).is_some_and(|rows| !rows.is_empty());

    let alert_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/schema/observe/prometheus-rule.schema.json"))
            .map_err(|err| format!("failed to read alert schema: {err}"))?,
    )
    .map_err(|err| format!("failed to parse alert schema: {err}"))?;
    let alert_contract: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/observe/contracts/alerts-contract.json"))
            .map_err(|err| format!("failed to read alert contract: {err}"))?,
    )
    .map_err(|err| format!("failed to parse alert contract: {err}"))?;
    let alerts_path = repo_root.join("ops/observe/alerts/atlas-alert-rules.yaml");
    let alert_rules: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(&alerts_path)
            .map_err(|err| format!("failed to read {}: {err}", alerts_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", alerts_path.display()))?;
    let groups = alert_rules
        .get("spec")
        .and_then(|row| row.get("groups"))
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut label_policy_passed = true;
    let mut alert_rules_reference_known_metrics = true;
    let label_policy: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../../configs/contracts/observability/label-policy.json"
    ))
    .map_err(|err| format!("failed to parse label policy: {err}"))?;
    let alert_required_labels = label_policy["alerts_required_labels"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let metric_required_labels = label_policy["metrics_required_labels"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    if !metric_required_labels.iter().all(|label| metrics_body.contains(&format!("{label}=\""))) {
        label_policy_passed = false;
    }
    let required_alerts = alert_contract
        .get("required_alerts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let mut observed_alerts = std::collections::BTreeSet::new();
    for group in &groups {
        let rules = group
            .get("rules")
            .and_then(serde_yaml::Value::as_sequence)
            .cloned()
            .unwrap_or_default();
        for rule in rules {
            if let Some(alert_name) = rule.get("alert").and_then(serde_yaml::Value::as_str) {
                observed_alerts.insert(alert_name.to_string());
            }
            let labels = rule
                .get("labels")
                .and_then(serde_yaml::Value::as_mapping)
                .cloned()
                .unwrap_or_default();
            for required in &alert_required_labels {
                let key = serde_yaml::Value::String((*required).to_string());
                if !labels.contains_key(&key) {
                    label_policy_passed = false;
                }
            }
            if let Some(expr) = rule.get("expr").and_then(serde_yaml::Value::as_str) {
                let known_metric = required_metric_names
                    .iter()
                    .any(|name| expr.contains(name))
                    || [
                        "atlas_overload_active",
                        "bijux_store_download_failure_total",
                        "bijux_dataset_hits",
                        "bijux_dataset_misses",
                    ]
                    .iter()
                    .any(|name| expr.contains(name));
                if !known_metric {
                    alert_rules_reference_known_metrics = false;
                }
            }
        }
    }
    let alert_rules_contract_valid = alert_schema
        .get("properties")
        .and_then(|row| row.get("kind"))
        .and_then(|row| row.get("const"))
        == Some(&serde_json::Value::String("PrometheusRule".to_string()))
        && alert_rules
            .get("kind")
            .and_then(serde_yaml::Value::as_str)
            == Some("PrometheusRule")
        && !groups.is_empty();
    if required_alerts.iter().any(|name| !observed_alerts.contains(*name)) {
        return Ok(serde_json::json!({
            "required_metrics_present": required_metrics_present,
            "missing_metrics": missing_metrics,
            "warmup_lock_metrics_present": warmup_lock_metrics_present,
            "error_registry_aligned": error_registry_aligned,
            "startup_log_fields_present": startup_log_fields_present,
            "redaction_contract_passed": redaction_contract_passed,
            "dashboard_contract_valid": dashboard_contract_valid,
            "slo_contract_valid": slo_contract_valid,
            "alert_rules_contract_valid": false,
            "alert_rules_reference_known_metrics": alert_rules_reference_known_metrics,
            "label_policy_passed": label_policy_passed
        }));
    }

    Ok(serde_json::json!({
        "required_metrics_present": required_metrics_present,
        "missing_metrics": missing_metrics,
        "warmup_lock_metrics_present": warmup_lock_metrics_present,
        "error_registry_aligned": error_registry_aligned,
        "startup_log_fields_present": startup_log_fields_present,
        "redaction_contract_passed": redaction_contract_passed,
        "dashboard_contract_valid": dashboard_contract_valid,
        "slo_contract_valid": slo_contract_valid,
        "alert_rules_contract_valid": alert_rules_contract_valid,
        "alert_rules_reference_known_metrics": alert_rules_reference_known_metrics,
        "label_policy_passed": label_policy_passed
    }))
}

pub(super) fn collect_report_paths(repo_root: &std::path::Path, run_id: &RunId) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for dir in [
        repo_root.join("ops/report/generated"),
        repo_root.join("artifacts/ops").join(run_id.as_str()).join("reports"),
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
            paths.push(path.strip_prefix(repo_root).unwrap_or(&path).display().to_string());
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub(super) fn collect_simulation_summary_paths(repo_root: &std::path::Path, run_id: &RunId) -> Vec<String> {
    let reports_dir = repo_root.join("artifacts/ops").join(run_id.as_str()).join("reports");
    ["ops-simulation-summary.json", "ops-lifecycle-summary.json"]
        .into_iter()
        .map(|name| reports_dir.join(name))
        .filter(|path| path.exists())
        .map(|path| path.strip_prefix(repo_root).unwrap_or(&path).display().to_string())
        .collect::<Vec<_>>()
}

pub(super) fn collect_drill_summary_paths(repo_root: &std::path::Path, run_id: &RunId) -> Vec<String> {
    let path = repo_root
        .join("artifacts/ops")
        .join(run_id.as_str())
        .join("reports")
        .join("ops-drills-summary.json");
    if path.exists() {
        vec![path.strip_prefix(repo_root).unwrap_or(&path).display().to_string()]
    } else {
        Vec::new()
    }
}

pub(super) fn collect_docs_site_summary(repo_root: &std::path::Path) -> Result<serde_json::Value, String> {
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

pub(super) fn collect_supply_chain_inventory(
    repo_root: &std::path::Path,
) -> Result<Vec<serde_json::Value>, String> {
    let paths = [
        ".github/dependabot.yml",
        "configs/docs/package-lock.json",
        "configs/docs/requirements.lock.txt",
        "configs/security/dependency-source-policy.json",
    ];
    let mut rows = Vec::new();
    for relative in paths {
        let path = repo_root.join(relative);
        if !path.exists() {
            return Err(format!("missing supply-chain inventory file {}", path.display()));
        }
        rows.push(serde_json::json!({
            "path": relative,
            "sha256": sha256_file(&path)?
        }));
    }
    Ok(rows)
}

pub(super) fn build_release_evidence_tarball(
    repo_root: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let evidence_root = evidence_root(repo_root)?;
    let tarball_path = evidence_root.join("bundle.tar");
    let list_path = evidence_root.join("bundle.list");
    let mut files = Vec::new();
    let mut stack = vec![evidence_root.clone()];
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            let Some(name) = entry_path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if name == "bundle.tar" || name == "bundle.list" {
                continue;
            }
            files.push(
                entry_path
                    .strip_prefix(repo_root)
                    .unwrap_or(&entry_path)
                    .display()
                    .to_string(),
            );
        }
    }
    files.extend(collect_observability_assets(repo_root)?);
    files.extend(collect_perf_assets(repo_root)?);
    files.extend(collect_dataset_assets(repo_root)?);
    files.extend(collect_governance_assets(repo_root)?);
    if repo_root.join("artifacts/security/security-github-actions.json").exists() {
        files.push("artifacts/security/security-github-actions.json".to_string());
    }
    if repo_root.join("artifacts/security/audit-verify.json").exists() {
        files.push("artifacts/security/audit-verify.json".to_string());
    }
    if repo_root.join("artifacts/security/audit-smoke.jsonl").exists() {
        files.push("artifacts/security/audit-smoke.jsonl".to_string());
    }
    if repo_root.join("artifacts/security/log-field-inventory.json").exists() {
        files.push("artifacts/security/log-field-inventory.json".to_string());
    }
    files.push("configs/security/auth-model.yaml".to_string());
    files.push("configs/security/policy.yaml".to_string());
    files.push("configs/observability/audit-log.schema.json".to_string());
    files.push("configs/observability/retention.yaml".to_string());
    files.push(".github/dependabot.yml".to_string());
    files.push("configs/docs/package-lock.json".to_string());
    files.push("configs/docs/requirements.lock.txt".to_string());
    files.push("configs/security/dependency-source-policy.json".to_string());
    files.sort();
    files.dedup();
    std::fs::write(&list_path, files.join("\n"))
        .map_err(|err| format!("failed to write {}: {err}", list_path.display()))?;
    let python = r#"import io, pathlib, tarfile
repo_root = pathlib.Path.cwd()
tarball_path = pathlib.Path(__import__("sys").argv[1])
list_path = pathlib.Path(__import__("sys").argv[2])
names = [line.strip() for line in list_path.read_text().splitlines() if line.strip()]
with tarfile.open(tarball_path, "w") as archive:
    for name in names:
        path = repo_root / name
        data = path.read_bytes()
        info = tarfile.TarInfo(name)
        info.size = len(data)
        info.mtime = 0
        info.uid = 0
        info.gid = 0
        info.uname = ""
        info.gname = ""
        info.mode = 0o644
        archive.addfile(info, io.BytesIO(data))
"#;
    let output = std::process::Command::new("python3")
        .current_dir(repo_root)
        .args([
            "-c",
            python,
            &tarball_path.display().to_string(),
            &list_path.display().to_string(),
        ])
        .output()
        .map_err(|err| format!("failed to execute tar for release evidence bundle: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to build release evidence tarball: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let _ = std::fs::remove_file(&list_path);
    Ok(tarball_path)
}

pub(super) fn tarball_contains_entry(
    tarball: &std::path::Path,
    entry_name: &str,
) -> Result<bool, String> {
    let output = std::process::Command::new("tar")
        .args(["-tf", &tarball.display().to_string()])
        .output()
        .map_err(|err| format!("failed to list {}: {err}", tarball.display()))?;
    if !output.status.success() {
        return Err(format!(
            "failed to list tarball {}: {}",
            tarball.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let listing = String::from_utf8_lossy(&output.stdout);
    let prefix = format!("{}/", entry_name.trim_end_matches('/'));
    Ok(listing.lines().any(|line| {
        let line = line.trim();
        line == entry_name || line.starts_with(&prefix)
    }))
}

pub(super) fn tarball_member_checksums(
    tarball: &std::path::Path,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let python = r#"import hashlib, json, pathlib, sys, tarfile
tarball_path = pathlib.Path(sys.argv[1])
rows = {}
with tarfile.open(tarball_path, "r") as archive:
    for member in archive.getmembers():
        if not member.isfile():
            continue
        extracted = archive.extractfile(member)
        if extracted is None:
            continue
        rows[member.name] = hashlib.sha256(extracted.read()).hexdigest()
print(json.dumps(rows, sort_keys=True))
"#;
    let output = std::process::Command::new("python3")
        .args(["-c", python, &tarball.display().to_string()])
        .output()
        .map_err(|err| format!("failed to inspect {}: {err}", tarball.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "failed to inspect {} members: {}",
            tarball.display(),
            if stderr.is_empty() {
                "python3 returned a non-zero exit status".to_string()
            } else {
                stderr
            }
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("failed to parse {} member checksums: {err}", tarball.display()))
}
