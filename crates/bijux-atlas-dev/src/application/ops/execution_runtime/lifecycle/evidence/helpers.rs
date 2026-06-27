// SPDX-License-Identifier: Apache-2.0
//! Shared evidence helpers for install-status flows.

use super::*;
use crate::OpsProcess;

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

fn extend_manifest_tarball_paths(
    repo_root: &std::path::Path,
    manifest: &serde_json::Value,
    files: &mut Vec<String>,
) {
    if let Some(action_pins_report) = manifest
        .get("supply_chain")
        .and_then(|value| value.get("action_pins_report"))
        .and_then(|value| value.get("path"))
        .and_then(serde_json::Value::as_str)
    {
        if repo_root.join(action_pins_report).exists() {
            files.push(action_pins_report.to_string());
        }
    }
    for rel in manifest
        .get("supply_chain")
        .and_then(|value| value.get("docs_toolchain_inventory"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
    {
        if repo_root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("reports")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .chain(
            manifest
                .get("simulation_summaries")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str),
        )
        .chain(
            manifest
                .get("drill_summaries")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str),
        )
        .chain(
            manifest
                .get("redacted_logs")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str),
        )
    {
        if repo_root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("ops_evidence")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|rows| rows.iter())
        .filter(|(name, _)| name.as_str() != "redaction_secret_keys")
        .filter_map(|(_, row)| row.get("path").and_then(serde_json::Value::as_str))
    {
        if repo_root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
}

pub(super) fn build_release_evidence_tarball(
    repo_root: &std::path::Path,
    manifest: &serde_json::Value,
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
    if repo_root
        .join("artifacts/security/security-github-actions.json")
        .exists()
    {
        files.push("artifacts/security/security-github-actions.json".to_string());
    }
    if repo_root
        .join("artifacts/security/audit-verify.json")
        .exists()
    {
        files.push("artifacts/security/audit-verify.json".to_string());
    }
    if repo_root
        .join("artifacts/security/audit-smoke.jsonl")
        .exists()
    {
        files.push("artifacts/security/audit-smoke.jsonl".to_string());
    }
    if repo_root
        .join("artifacts/security/log-field-inventory.json")
        .exists()
    {
        files.push("artifacts/security/log-field-inventory.json".to_string());
    }
    files.push("configs/sources/security/auth-model.yaml".to_string());
    files.push("configs/sources/security/policy.yaml".to_string());
    files
        .push("configs/sources/operations/observability/schemas/audit-log.schema.json".to_string());
    files.push("configs/sources/operations/observability/retention.yaml".to_string());
    files.push(".github/dependabot.yml".to_string());
    files.push("configs/sources/repository/docs/package-lock.json".to_string());
    files.push("configs/sources/repository/docs/requirements.lock.txt".to_string());
    files.push("configs/sources/security/dependency-source-policy.json".to_string());
    extend_manifest_tarball_paths(repo_root, manifest, &mut files);
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
    serde_json::from_slice(&output.stdout).map_err(|err| {
        format!(
            "failed to parse {} member checksums: {err}",
            tarball.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::extend_manifest_tarball_paths;

    #[test]
    fn manifest_ops_evidence_paths_are_added_to_bundle_members() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_root = temp.path();
        let rel = "artifacts/ops/evidence/ops_run/install-evidence.json";
        let abs = repo_root.join(rel);
        std::fs::create_dir_all(abs.parent().expect("parent")).expect("create parent");
        std::fs::write(&abs, "{}").expect("write artifact");
        let manifest = serde_json::json!({
            "ops_evidence": {
                "install_evidence": {
                    "path": rel
                },
                "redaction_secret_keys": ["token"]
            }
        });

        let mut files = Vec::new();
        extend_manifest_tarball_paths(repo_root, &manifest, &mut files);

        assert_eq!(files, vec![rel.to_string()]);
    }
}
