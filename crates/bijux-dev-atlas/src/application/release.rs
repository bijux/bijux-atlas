// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    FormatArg, ReleaseApiSurfaceCommand, ReleaseApiSurfaceSnapshotArgs, ReleaseBundleBuildArgs,
    ReleaseBundleHashArgs, ReleaseBundleVerifyArgs, ReleaseChangelogGenerateArgs,
    ReleaseChangelogValidateArgs, ReleaseCheckArgs, ReleaseCommand, ReleaseCompatibilityCheckArgs,
    ReleaseCratesCommand, ReleaseCratesDryRunArgs, ReleaseCratesListArgs,
    ReleaseCratesValidateArgs, ReleaseDiffArgs, ReleaseImagesChangelogArgs, ReleaseImagesCommand,
    ReleaseImagesIntegrationArgs, ReleaseImagesManifestArgs, ReleaseImagesNotesArgs,
    ReleaseImagesValidateArgs, ReleaseManifestGenerateArgs, ReleaseManifestValidateArgs,
    ReleaseMsrvCommand, ReleaseMsrvVerifyArgs, ReleaseOpsBundleArgs, ReleaseOpsCommand,
    ReleaseOpsPackageArgs, ReleaseOpsPullTestArgs, ReleaseOpsPushArgs, ReleasePacketArgs,
    ReleasePlanArgs, ReleaseRebuildVerifyArgs, ReleaseReproducibilityReportArgs,
    ReleaseSemverCheckArgs, ReleaseSemverCommand, ReleaseSignArgs, ReleaseTransitionPlanArgs,
    ReleaseValidateArgs, ReleaseVerifyArgs, ReleaseVersionCheckArgs,
};
use crate::{emit_payload, resolve_repo_root};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_yaml(path: &Path) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(
        &fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn env_var_text(key: &str) -> Option<String> {
    std::env::var_os(key).and_then(|value| value.into_string().ok())
}

fn ensure_json(path: &Path) -> Result<(), String> {
    let _: serde_json::Value = read_json(path)?;
    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value)
            .map_err(|err| format!("failed to encode {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn tarball_member_checksums(tarball: &Path) -> Result<BTreeMap<String, String>, String> {
    let python = r#"import hashlib, json, pathlib, sys, tarfile
tarball_path = pathlib.Path(sys.argv[1])
members = {}
with tarfile.open(tarball_path, "r") as archive:
    for member in archive.getmembers():
        if not member.isfile():
            continue
        handle = archive.extractfile(member)
        if handle is None:
            continue
        members[member.name] = hashlib.sha256(handle.read()).hexdigest()
print(json.dumps(members, sort_keys=True))
"#;
    let output = ProcessCommand::new("python3")
        .args(["-c", python, &tarball.display().to_string()])
        .output()
        .map_err(|err| format!("failed to inspect {} members: {err}", tarball.display()))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect {} members: {}",
            tarball.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|err| {
        format!(
            "failed to parse {} member checksums: {err}",
            tarball.display()
        )
    })
}

fn collect_tarball_members(
    root: &Path,
    manifest: &serde_json::Value,
) -> Result<Vec<String>, String> {
    let evidence_root = root.join("ops/release/evidence");
    let mut files = Vec::new();
    let mut stack = vec![evidence_root.clone()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)
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
            files.push(repo_rel(root, &entry_path));
        }
    }
    for rel in manifest
        .get("observability_assets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("perf_assets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("dataset_assets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        if root.join(rel).exists() {
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
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    if let Some(path) = manifest
        .get("supply_chain")
        .and_then(|v| v.get("action_pins_report"))
        .and_then(|v| v.get("path"))
        .and_then(serde_json::Value::as_str)
    {
        if root.join(path).exists() {
            files.push(path.to_string());
        }
    }
    for rel in manifest
        .get("audit_assets")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|rows| rows.values())
        .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
    {
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("governance_assets")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|rows| rows.values())
        .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
    {
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("auth_policy")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|rows| rows.values())
        .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
    {
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("supply_chain")
        .and_then(|v| v.get("docs_toolchain_inventory"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
    {
        if root.join(rel).exists() {
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
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    for rel in manifest
        .get("signature_artifacts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        if root.join(rel).exists() {
            files.push(rel.to_string());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn build_normalized_tarball(
    root: &Path,
    tarball_path: &Path,
    members: &[String],
) -> Result<(), String> {
    let python = r#"import io, pathlib, sys, tarfile
repo_root = pathlib.Path.cwd()
tarball_path = pathlib.Path(sys.argv[1])
names = sys.argv[2:]
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
    let mut args = vec![
        "-c".to_string(),
        python.to_string(),
        tarball_path.display().to_string(),
    ];
    args.extend(members.iter().cloned());
    let output = ProcessCommand::new("python3")
        .current_dir(root)
        .args(&args)
        .output()
        .map_err(|err| format!("failed to execute normalized tar build: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to build normalized tarball: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

fn run_release_check(args: ReleaseCheckArgs) -> Result<(String, i32), String> {
    let exe = std::env::current_exe().map_err(|err| format!("release check failed: {err}"))?;

    let mut validate_args = vec![
        "validate".to_string(),
        "--profile".to_string(),
        args.profile.clone(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if let Some(root) = &args.repo_root {
        validate_args.push("--repo-root".to_string());
        validate_args.push(root.display().to_string());
    }
    let validate_out = ProcessCommand::new(&exe)
        .args(&validate_args)
        .output()
        .map_err(|err| format!("release check failed: {err}"))?;
    let validate_payload: serde_json::Value = serde_json::from_slice(&validate_out.stdout).unwrap_or_else(|_| {
        serde_json::json!({"status":"failed","stderr": String::from_utf8_lossy(&validate_out.stderr)})
    });

    let mut readiness_args = vec![
        "ops".to_string(),
        "validate".to_string(),
        "--profile".to_string(),
        args.profile.clone(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if let Some(root) = &args.repo_root {
        readiness_args.push("--repo-root".to_string());
        readiness_args.push(root.display().to_string());
    }
    let readiness_out = ProcessCommand::new(&exe)
        .args(&readiness_args)
        .output()
        .map_err(|err| format!("release check failed: {err}"))?;
    let readiness_payload: serde_json::Value =
        serde_json::from_slice(&readiness_out.stdout).unwrap_or_else(|_| {
            serde_json::json!({"status":"failed","stderr": String::from_utf8_lossy(&readiness_out.stderr)})
        });

    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_reproducibility_policy(&root)?;
    let evidence_rel = policy
        .get("evidence_report_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("artifacts/release/reproducibility-report.json");
    let evidence_path = root.join(evidence_rel);
    let evidence_payload = if evidence_path.exists() {
        read_json(&evidence_path).unwrap_or_else(|_| serde_json::json!({"status":"failed"}))
    } else {
        serde_json::json!({
            "status": "failed",
            "errors": ["missing reproducibility evidence report"]
        })
    };
    let require_evidence = policy
        .get("require_evidence_before_release")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let evidence_ok = !require_evidence
        || evidence_payload
            .get("status")
            .and_then(serde_json::Value::as_str)
            == Some("ok");
    let ok = validate_out.status.success() && readiness_out.status.success() && evidence_ok;
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if ok { "ok" } else { "failed" },
        "text": if ok { "release check passed" } else { "release check failed" },
        "validate": validate_payload,
        "ops_validate": readiness_payload,
        "reproducibility_evidence": {
            "path": evidence_rel,
            "status": evidence_payload.get("status").cloned().unwrap_or(serde_json::json!("failed")),
            "report": evidence_payload
        }
    });
    let rendered = match args.format {
        FormatArg::Json => {
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        }
        FormatArg::Text => {
            if ok {
                "release check passed: validate + ops validate".to_string()
            } else {
                "release check failed: rerun with --format json for details".to_string()
            }
        }
        FormatArg::Jsonl => payload.to_string(),
    };
    if let Some(path) = args.out {
        fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("release check failed: {err}"))?;
    }
    Ok((rendered, if ok { 0 } else { 1 }))
}

fn read_publish_policy(root: &Path) -> Result<serde_json::Value, String> {
    read_json(&root.join("configs/sources/release/publish-policy.json"))
}

fn read_crates_release_spec(root: &Path) -> Result<toml::Value, String> {
    let path = root.join("ops/release/crates-release.toml");
    toml::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn read_images_release_spec(root: &Path) -> Result<toml::Value, String> {
    let path = root.join("ops/release/images-release.toml");
    toml::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn dockerfile_label_map(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut labels = BTreeMap::new();
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("LABEL ") {
            continue;
        }
        let rest = trimmed.trim_start_matches("LABEL ").trim();
        if let Some((k, v)) = rest.split_once('=') {
            labels.insert(
                k.trim().to_ascii_lowercase(),
                v.trim().trim_matches('"').to_string(),
            );
        }
    }
    Ok(labels)
}

fn dockerfile_from_rows(path: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("FROM "))
        .map(|line| {
            line.trim_start_matches("FROM ")
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string()
        })
        .collect())
}

fn run_release_images_validate_labels(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_json(&root.join("ops/docker/policy.json"))?;
    let required = policy
        .get("required_oci_labels")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.as_str().map(|v| v.to_ascii_lowercase()))
        .collect::<Vec<_>>();
    let dockerfile = root.join("ops/docker/images/runtime/Dockerfile");
    let labels = dockerfile_label_map(&dockerfile)?;
    let mut errors = Vec::<String>::new();
    for key in &required {
        match labels.get(key) {
            None => errors.push(format!("runtime Dockerfile missing label `{key}`")),
            Some(value) if value.trim().is_empty() => errors.push(format!(
                "runtime Dockerfile label `{key}` must be non-empty"
            )),
            _ => {}
        }
    }
    for required in [
        "org.opencontainers.image.version",
        "org.opencontainers.image.revision",
        "org.opencontainers.image.source",
        "org.opencontainers.image.created",
        "org.opencontainers.image.licenses",
    ] {
        if !labels.contains_key(required) {
            errors.push(format!(
                "runtime Dockerfile missing release label `{required}`"
            ));
        }
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_validate_labels",
        "status": status,
        "dockerfile": repo_rel(&root, &dockerfile),
        "required_labels": required,
        "present_labels": labels.keys().cloned().collect::<Vec<_>>(),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_validate_tags(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_images_release_spec(&root)?;
    let forbid_latest = spec
        .get("policy")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("forbid_latest_tag"))
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let mut rows = Vec::<serde_json::Value>::new();
    let mut errors = Vec::<String>::new();
    for image in spec
        .get("images")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        let name = image
            .get("name")
            .and_then(toml::Value::as_str)
            .unwrap_or("unknown");
        let tag_policy = image
            .get("tag_policy")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>();
        let allow_latest = image
            .get("allow_latest")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        if forbid_latest && allow_latest {
            errors.push(format!(
                "image `{name}` allows latest while policy forbids it"
            ));
        }
        if forbid_latest
            && tag_policy
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case("latest") || tag.contains(":latest"))
        {
            errors.push(format!(
                "image `{name}` tag policy includes forbidden latest reference"
            ));
        }
        if tag_policy.is_empty() {
            errors.push(format!("image `{name}` is missing tag policy"));
        }
        rows.push(serde_json::json!({
            "name": name,
            "tag_policy": tag_policy,
            "allow_latest": allow_latest
        }));
    }
    rows.sort_by(|a, b| {
        a["name"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["name"].as_str().unwrap_or_default())
    });
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_validate_tags",
        "status": status,
        "forbid_latest": forbid_latest,
        "images": rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_validate_base_digests(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_images_release_spec(&root)?;
    let pinning = spec
        .get("base_image_pinning")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let require_digest_pins = pinning
        .get("require_digest_pins")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let lockfile = pinning
        .get("lockfile")
        .and_then(toml::Value::as_str)
        .unwrap_or("ops/docker/bases.lock");
    let lock = read_json(&root.join(lockfile))?;
    let lock_rows = lock
        .get("images")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let lock_map = lock_rows
        .into_iter()
        .filter_map(|row| {
            let image = row.get("image").and_then(serde_json::Value::as_str)?;
            let digest = row.get("digest").and_then(serde_json::Value::as_str)?;
            Some((image.to_string(), digest.to_string()))
        })
        .collect::<BTreeMap<_, _>>();
    let dockerfile = root.join("ops/docker/images/runtime/Dockerfile");
    let mut rows = Vec::<serde_json::Value>::new();
    let mut errors = Vec::<String>::new();
    for from in dockerfile_from_rows(&dockerfile)? {
        let (image_ref, digest) = if let Some((base, digest)) = from.split_once('@') {
            (base.to_string(), Some(digest.to_string()))
        } else {
            (from.clone(), None)
        };
        if require_digest_pins && digest.is_none() {
            errors.push(format!("FROM `{from}` is missing digest pin"));
        }
        if let Some(found) = lock_map.get(&image_ref) {
            let actual = digest
                .as_deref()
                .map(|v| format!("@{v}"))
                .unwrap_or_default();
            if actual != format!("@{found}") {
                errors.push(format!(
                    "FROM `{from}` digest does not match lockfile `{image_ref}@{found}`"
                ));
            }
        }
        rows.push(serde_json::json!({
            "from": from,
            "image": image_ref,
            "digest": digest
        }));
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_validate_base_digests",
        "status": status,
        "lockfile": lockfile,
        "require_digest_pins": require_digest_pins,
        "dockerfile": repo_rel(&root, &dockerfile),
        "rows": rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_sbom_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_json(&root.join("configs/sources/release/image-sbom-policy.json"))?;
    let required = policy
        .get("required")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let formats = policy
        .get("formats")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let required_artifacts = policy
        .get("required_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let schema_reference = policy
        .get("schema_reference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let mut errors = Vec::<String>::new();
    if schema_reference.is_empty() || !root.join(schema_reference).exists() {
        errors.push("SBOM schema reference is missing or does not exist".to_string());
    }
    let mut rows = Vec::<serde_json::Value>::new();
    for artifact in &required_artifacts {
        let Some(path) = artifact.as_str() else {
            continue;
        };
        let full = root.join(path);
        let exists = full.exists();
        if required && !exists {
            errors.push(format!("required SBOM artifact missing: {path}"));
        }
        rows.push(serde_json::json!({
            "path": path,
            "exists": exists
        }));
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_sbom_verify",
        "status": status,
        "required": required,
        "formats": formats,
        "schema_reference": schema_reference,
        "artifacts": rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_provenance_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_json(&root.join("configs/sources/release/image-provenance-policy.json"))?;
    let required = policy
        .get("required")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    let schema_reference = policy
        .get("schema_reference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let required_artifacts = policy
        .get("required_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let push_policy = read_json(&root.join("ops/docker/push-policy.json"))?;
    let require_provenance_bundle = push_policy
        .get("require_provenance_bundle")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut errors = Vec::<String>::new();
    if required && !require_provenance_bundle {
        errors.push("ops/docker/push-policy.json must require provenance bundle".to_string());
    }
    if schema_reference.is_empty() || !root.join(schema_reference).exists() {
        errors.push("provenance schema reference is missing or does not exist".to_string());
    }
    let mut rows = Vec::<serde_json::Value>::new();
    for artifact in &required_artifacts {
        let Some(path) = artifact.as_str() else {
            continue;
        };
        let exists = root.join(path).exists();
        if required && !exists {
            errors.push(format!("required provenance artifact missing: {path}"));
        }
        rows.push(serde_json::json!({"path": path, "exists": exists}));
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_provenance_verify",
        "status": status,
        "required": required,
        "schema_reference": schema_reference,
        "push_policy_requires_bundle": require_provenance_bundle,
        "artifacts": rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_scan_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_json(&root.join("configs/sources/release/image-vulnerability-policy.json"))?;
    let required_artifacts = policy
        .get("required_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::<String>::new();
    let mut rows = Vec::<serde_json::Value>::new();
    for artifact in required_artifacts {
        let Some(path) = artifact.as_str() else {
            continue;
        };
        let exists = root.join(path).exists();
        if !exists {
            errors.push(format!("required vulnerability artifact missing: {path}"));
        }
        rows.push(serde_json::json!({"path": path, "exists": exists}));
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_scan_verify",
        "status": status,
        "mode": policy.get("mode").cloned().unwrap_or(serde_json::Value::String("informational".to_string())),
        "severity_budget": policy.get("severity_budget").cloned().unwrap_or(serde_json::Value::Null),
        "artifacts": rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_smoke_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_json(&root.join("configs/sources/release/image-smoke-scenarios.json"))?;
    let checks = spec
        .get("required_checks")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::<String>::new();
    for required in ["healthz", "readyz", "version", "config_parse_error"] {
        let found = checks.iter().any(|row| {
            row.get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == required)
        });
        if !found {
            errors.push(format!("smoke scenario is missing check `{required}`"));
        }
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_smoke_verify",
        "status": status,
        "scenario": spec.get("scenario").cloned().unwrap_or(serde_json::Value::String("runtime-minimal-config".to_string())),
        "checks": checks,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_size_report(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy =
        read_json(&root.join("configs/sources/release/image-runtime-hardening-policy.json"))?;
    let budget = policy
        .get("runtime_image_max_bytes")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(450_000_000);
    let binary = root.join("artifacts/target/ops/release/bijux-atlas-server");
    let binary_bytes = if binary.exists() {
        Some(
            fs::metadata(&binary)
                .map_err(|err| format!("failed to stat {}: {err}", binary.display()))?
                .len(),
        )
    } else {
        None
    };
    let measured_image_size_path = root.join("artifacts/docker-publish/image-size-bytes.json");
    let measured_image_size = if measured_image_size_path.exists() {
        Some(
            read_json(&measured_image_size_path)?
                .get("bytes")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default(),
        )
    } else {
        None
    };
    let waiver_path = policy
        .get("size_waiver_path")
        .and_then(serde_json::Value::as_str)
        .map(|rel| root.join(rel));
    let waiver_present = waiver_path.as_ref().is_some_and(|path| path.exists());
    let mut warnings = Vec::<String>::new();
    let mut errors = Vec::<String>::new();
    if let Some(actual) = measured_image_size {
        if actual > budget && !waiver_present {
            errors.push(format!(
                "runtime image size {} exceeds budget {} bytes without waiver",
                actual, budget
            ));
        }
    } else {
        warnings.push(
            "missing artifacts/docker-publish/image-size-bytes.json; measured image budget check skipped"
                .to_string(),
        );
    }
    let status = if errors.is_empty() {
        if warnings.is_empty() {
            "ok"
        } else {
            "warn"
        }
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_size_report",
        "status": status,
        "runtime_image_budget_bytes": budget,
        "atlas_server_binary_bytes": binary_bytes,
        "measured_image_size_bytes": measured_image_size,
        "waiver_present": waiver_present,
        "note": "budget tracks image bytes; binary size is included as a local deterministic proxy signal",
        "warnings": warnings,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "failed" { 1 } else { 0 }))
}

fn run_release_images_runtime_hardening_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy =
        read_json(&root.join("configs/sources/release/image-runtime-hardening-policy.json"))?;
    let dockerfile = root.join("ops/docker/images/runtime/Dockerfile");
    let text = fs::read_to_string(&dockerfile)
        .map_err(|err| format!("failed to read {}: {err}", dockerfile.display()))?;
    let mut errors = Vec::<String>::new();
    let runtime_section = text
        .split("\nFROM ")
        .find(|chunk| chunk.contains(" AS runtime") || chunk.contains(" as runtime"))
        .unwrap_or(text.as_str())
        .to_string();
    if policy
        .get("require_non_root")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
        && !runtime_section.contains("USER nonroot:nonroot")
    {
        errors.push("runtime image must run as non-root user".to_string());
    }
    if policy
        .get("require_minimal_runtime_base")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
        && !runtime_section.contains("distroless")
    {
        errors.push("runtime base image must be minimal (distroless expected)".to_string());
    }
    let forbidden_tokens = policy
        .get("forbidden_runtime_tokens")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for token in forbidden_tokens {
        let Some(token_text) = token.as_str() else {
            continue;
        };
        if runtime_section.contains(token_text) {
            errors.push(format!(
                "runtime Dockerfile contains forbidden token `{token_text}`"
            ));
        }
    }
    if policy
        .get("require_minimal_write_surface")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
        && (runtime_section.contains("chmod 777") || runtime_section.contains("chown -R"))
    {
        errors.push(
            "runtime Dockerfile must avoid broad write-surface permission grants".to_string(),
        );
    }
    let allowed_runtime_binaries = policy
        .get("allowed_runtime_binaries")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let runtime_copy_targets = runtime_section
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("COPY --from=builder "))
        .filter_map(|line| line.split_whitespace().last())
        .collect::<Vec<_>>();
    for target in runtime_copy_targets {
        if !allowed_runtime_binaries
            .iter()
            .any(|allowed| allowed == target)
        {
            errors.push(format!(
                "runtime COPY target `{target}` is not listed in allowed_runtime_binaries"
            ));
        }
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_runtime_hardening_verify",
        "status": status,
        "dockerfile": repo_rel(&root, &dockerfile),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_runtime_command_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let dev_cli_dispatch = crate::reference::workspace_layout::dev_atlas_cli_dispatch_source(&root);
    let dev_cli_mod = crate::reference::workspace_layout::dev_atlas_cli_mod_source(&root);
    let schema_path = crate::reference::workspace_layout::atlas_runtime_generated_artifact(
        &root,
        "runtime-startup-config.schema.json",
    );
    let dispatch_text = fs::read_to_string(&dev_cli_dispatch)
        .map_err(|err| format!("failed to read {}: {err}", dev_cli_dispatch.display()))?;
    let cli_text = fs::read_to_string(&dev_cli_mod)
        .map_err(|err| format!("failed to read {}: {err}", dev_cli_mod.display()))?;
    let mut errors = Vec::<String>::new();
    if !cli_text.contains("PrintConfigSchema(RuntimeCommandArgs)") {
        errors
            .push("dev CLI is missing runtime print-config-schema command declaration".to_string());
    }
    if !dispatch_text.contains("Command::Runtime { command }") {
        errors.push("dev CLI dispatcher is missing Runtime command route".to_string());
    }
    if !cli_text.contains("SelfCheck(RuntimeCommandArgs)") {
        errors.push("dev CLI is missing runtime self-check command declaration".to_string());
    }
    if !cli_text.contains("ExplainConfigSchema(RuntimeCommandArgs)") {
        errors.push(
            "dev CLI is missing runtime explain-config-schema command declaration".to_string(),
        );
    }
    if !schema_path.exists() {
        errors.push(format!(
            "runtime config schema is missing: {}",
            schema_path.display()
        ));
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_runtime_command_verify",
        "status": status,
        "checks": [
            {"id":"runtime_print_config_schema_command","status": if cli_text.contains("PrintConfigSchema(RuntimeCommandArgs)") {"ok"} else {"failed"}},
            {"id":"runtime_self_check_command","status": if cli_text.contains("SelfCheck(RuntimeCommandArgs)") {"ok"} else {"failed"}},
            {"id":"runtime_explain_config_schema_command","status": if cli_text.contains("ExplainConfigSchema(RuntimeCommandArgs)") {"ok"} else {"failed"}},
            {"id":"runtime_schema_reference_exists","status": if schema_path.exists() {"ok"} else {"failed"}},
            {"id":"runtime_self_check_offline_contract","status": if dispatch_text.contains("Command::Runtime { command }") {"ok"} else {"failed"}}
        ],
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_manifest_generate(
    args: ReleaseImagesManifestArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    if !args.allow_write {
        return Err("release images manifest-generate requires --allow-write".to_string());
    }
    let digests_path = root.join("artifacts/docker-publish/image-digests.json");
    let digests = read_json(&digests_path)?;
    let docker_manifest_path = root.join("ops/docker/images.manifest.json");
    let docker_manifest = read_json(&docker_manifest_path)?;
    let out_path = root.join("ops/release/images/image-artifact-manifest.v0.1.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_image_artifact_manifest",
        "generated_at_utc": "1970-01-01T00:00:00Z",
        "sources": {
            "image_digests": repo_rel(&root, &digests_path),
            "docker_manifest": repo_rel(&root, &docker_manifest_path)
        },
        "image_digests": digests.get("images").cloned().unwrap_or(serde_json::Value::Null),
        "declared_images": docker_manifest.get("images").cloned().unwrap_or(serde_json::Value::Null)
    });
    write_json(&out_path, &payload)?;
    let rendered_payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_manifest_generate",
        "status": "ok",
        "path": repo_rel(&root, &out_path),
        "sha256": sha256_file(&out_path)?
    });
    let rendered = emit_payload(args.format, args.out, &rendered_payload)?;
    Ok((rendered, 0))
}

fn run_release_images_manifest_verify(
    args: ReleaseImagesManifestArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let manifest_path = root.join("ops/release/images/image-artifact-manifest.v0.1.json");
    let manifest = read_json(&manifest_path)?;
    let mut errors = Vec::<String>::new();
    let image_digests = manifest
        .get("image_digests")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let declared = manifest
        .get("declared_images")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let declared_set = declared
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    for row in &image_digests {
        let Some(tag) = row.get("tag").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(digest) = row.get("digest").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if !digest.starts_with("sha256:") {
            errors.push(format!("image digest must be sha256-prefixed: {tag}"));
        }
        let image_name = tag.split(':').next().unwrap_or_default();
        if !declared_set.contains(image_name) {
            errors.push(format!(
                "image tag `{tag}` is not declared in ops/docker/images.manifest.json"
            ));
        }
    }
    let registry_path = root.join("ops/release/image-digest-registry.json");
    if registry_path.exists() {
        let registry = read_json(&registry_path)?;
        let immutables = registry
            .get("released")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        for row in immutables {
            let Some(tag) = row.get("tag").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(digest) = row.get("digest").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let changed = image_digests.iter().any(|candidate| {
                candidate.get("tag").and_then(serde_json::Value::as_str) == Some(tag)
                    && candidate.get("digest").and_then(serde_json::Value::as_str) != Some(digest)
            });
            if changed {
                errors.push(format!(
                    "digest registry immutability violation for tag `{tag}`"
                ));
            }
        }
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_manifest_verify",
        "status": status,
        "manifest_path": repo_rel(&root, &manifest_path),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_release_notes_check(
    args: ReleaseImagesNotesArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let version = args
        .version
        .unwrap_or_else(|| workspace_version(&root).unwrap_or_else(|_| "0.1.0".to_string()));
    let template_path = root.join("ops/release/notes/image-release-notes-template.txt");
    let notes_path = root.join(format!("ops/release/notes/images/{version}.txt"));
    let mut errors = Vec::<String>::new();
    if !template_path.exists() {
        errors.push("image release notes template is missing".to_string());
    }
    if !notes_path.exists() {
        errors.push(format!(
            "image release notes missing for version `{version}` at {}",
            notes_path.display()
        ));
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_release_notes_check",
        "status": status,
        "version": version,
        "template_path": repo_rel(&root, &template_path),
        "notes_path": repo_rel(&root, &notes_path),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_changelog_extract(
    args: ReleaseImagesChangelogArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let from_ref = args.from_ref.unwrap_or_else(|| "HEAD~30".to_string());
    let to_ref = args.to_ref.unwrap_or_else(|| "HEAD".to_string());
    let version = args
        .version
        .unwrap_or_else(|| workspace_version(&root).unwrap_or_else(|_| "0.1.0".to_string()));
    let output = ProcessCommand::new("git")
        .args([
            "log",
            "--oneline",
            &format!("{from_ref}..{to_ref}"),
            "--",
            "docker",
            "crates/bijux-atlas",
            "crates/bijux-atlas",
        ])
        .current_dir(&root)
        .output()
        .map_err(|err| format!("failed to run git log: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let lines = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let out_path = root.join(format!("ops/release/notes/images/changelog-{version}.txt"));
    if args.allow_write {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let mut body = format!("Image Changelog {version}\n\n");
        for line in &lines {
            body.push_str("- ");
            body.push_str(line);
            body.push('\n');
        }
        fs::write(&out_path, body)
            .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_changelog_extract",
        "status": "ok",
        "version": version,
        "from_ref": from_ref,
        "to_ref": to_ref,
        "entries": lines,
        "output_path": repo_rel(&root, &out_path),
        "wrote_file": args.allow_write
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn read_ops_release_spec(root: &Path) -> Result<toml::Value, String> {
    let path = root.join("ops/release/ops-release.toml");
    toml::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn run_release_images_integration_verify(
    args: ReleaseImagesIntegrationArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let image_spec = read_images_release_spec(&root)?;
    let image_name = image_spec
        .get("images")
        .and_then(toml::Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("name"))
        .and_then(toml::Value::as_str)
        .unwrap_or("ghcr.io/bijux/bijux-atlas-server");
    let digests_path = root.join("artifacts/docker-publish/image-digests.json");
    let digest_payload = if digests_path.exists() {
        Some(read_json(&digests_path)?)
    } else {
        None
    };
    let digest = digest_payload
        .as_ref()
        .and_then(|v| v.get("images"))
        .and_then(serde_json::Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("digest"))
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    let deploy_reference = if args.by_digest {
        if let Some(digest) = &digest {
            format!("{image_name}@{digest}")
        } else {
            format!("{image_name}:v0.1.0")
        }
    } else {
        format!("{image_name}:v0.1.0")
    };
    let evidence_path = root.join("ops/release/evidence/image-ops-integration.json");
    if args.allow_write {
        let payload = serde_json::json!({
            "schema_version": 1,
            "scenario": "image-ops-deploy-kind",
            "status": "simulated",
            "deployment_reference": deploy_reference,
            "image_digest": digest,
            "chart_reference": "ops/k8s/charts/bijux-atlas",
            "evidence_contract": {
                "includes_image_digest": true
            }
        });
        write_json(&evidence_path, &payload)?;
    }
    let mut errors = Vec::<String>::new();
    if args.by_digest && digest.is_none() {
        errors.push("digest deployment requested but artifacts/docker-publish/image-digests.json is missing".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_integration_verify",
        "status": status,
        "deployment_reference": deploy_reference,
        "evidence_path": repo_rel(&root, &evidence_path),
        "image_digest": digest,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_build_reproducibility_check(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let dockerfile = root.join("ops/docker/images/runtime/Dockerfile");
    let text = fs::read_to_string(&dockerfile)
        .map_err(|err| format!("failed to read {}: {err}", dockerfile.display()))?;
    let mut errors = Vec::<String>::new();
    for token in [
        "ARG BUILD_DATE",
        "ARG SOURCE_DATE_EPOCH",
        "ARG VCS_REF",
        "ARG IMAGE_VERSION",
    ] {
        if !text.contains(token) {
            errors.push(format!(
                "Dockerfile missing reproducibility build arg `{token}`"
            ));
        }
    }
    if !text.contains("org.opencontainers.image.created=\"${BUILD_DATE}\"") {
        errors.push("Dockerfile label must bind created timestamp to BUILD_DATE".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_build_reproducibility_check",
        "status": status,
        "dockerfile": repo_rel(&root, &dockerfile),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_locked_dependencies_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let dockerfile = root.join("ops/docker/images/runtime/Dockerfile");
    let text = fs::read_to_string(&dockerfile)
        .map_err(|err| format!("failed to read {}: {err}", dockerfile.display()))?;
    let mut errors = Vec::<String>::new();
    if !text.contains("cargo build --locked") {
        errors.push("Dockerfile must build with cargo --locked".to_string());
    }
    if !root.join("Cargo.lock").exists() {
        errors.push("Cargo.lock is missing".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_locked_dependencies_verify",
        "status": status,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_lock_drift_verify(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let output = ProcessCommand::new("cargo")
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(&root)
        .output()
        .map_err(|err| format!("failed to run cargo metadata --locked: {err}"))?;
    let status = if output.status.success() {
        "ok"
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_lock_drift_verify",
        "status": status,
        "stderr": String::from_utf8_lossy(&output.stderr).to_string()
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_images_readiness_summary(
    args: ReleaseImagesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let checks = [
        (
            "labels",
            run_release_images_validate_labels(ReleaseImagesValidateArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
            })
            .map(|(_, c)| c == 0)
            .unwrap_or(false),
        ),
        (
            "tags",
            run_release_images_validate_tags(ReleaseImagesValidateArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
            })
            .map(|(_, c)| c == 0)
            .unwrap_or(false),
        ),
        (
            "base_digests",
            run_release_images_validate_base_digests(ReleaseImagesValidateArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
            })
            .map(|(_, c)| c == 0)
            .unwrap_or(false),
        ),
        (
            "runtime_hardening",
            run_release_images_runtime_hardening_verify(ReleaseImagesValidateArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
            })
            .map(|(_, c)| c == 0)
            .unwrap_or(false),
        ),
        (
            "runtime_commands",
            run_release_images_runtime_command_verify(ReleaseImagesValidateArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
            })
            .map(|(_, c)| c == 0)
            .unwrap_or(false),
        ),
        (
            "locked_dependencies",
            run_release_images_locked_dependencies_verify(ReleaseImagesValidateArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
            })
            .map(|(_, c)| c == 0)
            .unwrap_or(false),
        ),
    ];
    let rows = checks
        .iter()
        .map(|(name, ok)| serde_json::json!({"check": name, "status": if *ok { "ok" } else { "failed" }}))
        .collect::<Vec<_>>();
    let passed = checks.iter().filter(|(_, ok)| *ok).count();
    let total = checks.len();
    let status = if passed == total { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_images_readiness_summary",
        "status": status,
        "passed": passed,
        "total": total,
        "rows": rows
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_package(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    if !args.allow_write {
        return Err("release ops package requires --allow-write".to_string());
    }
    let spec = read_ops_release_spec(&root)?;
    let chart_rel = spec
        .get("chart")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("path"))
        .and_then(toml::Value::as_str)
        .unwrap_or("ops/k8s/charts/bijux-atlas");
    let out_rel = spec
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("output_dir"))
        .and_then(toml::Value::as_str)
        .unwrap_or("artifacts/release/ops/package");
    let chart_path = root.join(chart_rel);
    let out_dir = root.join(out_rel);
    fs::create_dir_all(&out_dir)
        .map_err(|err| format!("failed to create {}: {err}", out_dir.display()))?;
    let mut helm_package_stdout = String::new();
    let mut helm_lint_stdout = String::new();
    let mut errors = Vec::<String>::new();
    if args.allow_subprocess {
        let lint = ProcessCommand::new("helm")
            .args(["lint", chart_path.to_string_lossy().as_ref()])
            .current_dir(&root)
            .output()
            .map_err(|err| format!("failed to run helm lint: {err}"))?;
        helm_lint_stdout = String::from_utf8_lossy(&lint.stdout).to_string();
        if !lint.status.success() {
            errors.push(format!(
                "helm lint failed: {}",
                String::from_utf8_lossy(&lint.stderr).trim()
            ));
        }
        let package = ProcessCommand::new("helm")
            .args([
                "package",
                chart_path.to_string_lossy().as_ref(),
                "--destination",
                out_dir.to_string_lossy().as_ref(),
            ])
            .current_dir(&root)
            .output()
            .map_err(|err| format!("failed to run helm package: {err}"))?;
        helm_package_stdout = String::from_utf8_lossy(&package.stdout).to_string();
        if !package.status.success() {
            errors.push(format!(
                "helm package failed: {}",
                String::from_utf8_lossy(&package.stderr).trim()
            ));
        }
    } else {
        errors.push(
            "release ops package requires --allow-subprocess for helm package/lint".to_string(),
        );
    }
    let packages = fs::read_dir(&out_dir)
        .map_err(|err| format!("failed to read {}: {err}", out_dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("tgz"))
        .collect::<Vec<_>>();
    let package_rows = packages
        .iter()
        .map(|path| {
            serde_json::json!({
                "path": repo_rel(&root, path),
                "sha256": sha256_file(path).ok(),
            })
        })
        .collect::<Vec<_>>();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_package",
        "status": status,
        "chart_path": chart_rel,
        "output_dir": out_rel,
        "helm_lint_stdout": helm_lint_stdout,
        "helm_package_stdout": helm_package_stdout,
        "packages": package_rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_validate_package(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_ops_release_spec(&root)?;
    let out_rel = spec
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("output_dir"))
        .and_then(toml::Value::as_str)
        .unwrap_or("artifacts/release/ops/package");
    let out_dir = root.join(out_rel);
    let mut errors = Vec::<String>::new();
    let chart_tgz = if out_dir.exists() {
        fs::read_dir(&out_dir)
            .map_err(|err| format!("failed to read {}: {err}", out_dir.display()))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .find(|p| p.extension().and_then(|e| e.to_str()) == Some("tgz"))
    } else {
        errors.push(format!(
            "ops package output directory missing: {}",
            out_dir.display()
        ));
        None
    };
    if chart_tgz.is_none() {
        errors.push(format!("no chart package found in {}", out_dir.display()));
    }
    let chart_path = root.join("ops/k8s/charts/bijux-atlas");
    if !chart_path.join("values.schema.json").exists() {
        errors.push("chart values.schema.json is missing".to_string());
    }
    if !chart_path.join("Chart.yaml").exists() {
        errors.push("chart Chart.yaml is missing".to_string());
    }
    let evidence_marker = root.join("ops/release/evidence/ops-install-evidence-bundle.json");
    if !evidence_marker.exists() {
        errors.push("ops install evidence bundle is missing: ops/release/evidence/ops-install-evidence-bundle.json".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_validate_package",
        "status": status,
        "output_dir": out_rel,
        "chart_package_path": chart_tgz.as_ref().map(|p| repo_rel(&root, p)),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn find_ops_chart_tgz(out_dir: &Path) -> Result<Option<std::path::PathBuf>, String> {
    if !out_dir.exists() {
        return Ok(None);
    }
    let mut rows = fs::read_dir(out_dir)
        .map_err(|err| format!("failed to read {}: {err}", out_dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("tgz"))
        .collect::<Vec<_>>();
    rows.sort();
    Ok(rows.pop())
}

fn release_version_from_chart(chart_yaml: &Path) -> Result<String, String> {
    let yaml = read_yaml(chart_yaml)?;
    yaml.get("version")
        .and_then(serde_yaml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("missing `version` in {}", chart_yaml.display()))
}

fn run_release_ops_push(args: ReleaseOpsPushArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.common.repo_root.clone())?;
    let spec = read_ops_release_spec(&root)?;
    let out_rel = spec
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("output_dir"))
        .and_then(toml::Value::as_str)
        .unwrap_or("artifacts/release/ops/package");
    let out_dir = root.join(out_rel);
    let chart_pkg = find_ops_chart_tgz(&out_dir)?;
    let chart_pkg = match chart_pkg {
        Some(path) => path,
        None => {
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "release_ops_push",
                "status": "failed",
                "errors": [format!("no packaged chart found in {}", out_dir.display())]
            });
            let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
            return Ok((rendered, 1));
        }
    };
    let chart_ref = spec
        .get("distribution")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("chart_reference"))
        .and_then(toml::Value::as_str)
        .unwrap_or("oci://ghcr.io/bijux/charts/bijux-atlas")
        .to_string();
    let mut errors = Vec::<String>::new();
    let mut stdout = String::new();
    if args.dry_run {
        stdout = format!("dry-run: helm push {} {}", chart_pkg.display(), chart_ref);
    } else if args.common.allow_subprocess && args.allow_network {
        let output = ProcessCommand::new("helm")
            .args([
                "push",
                chart_pkg.to_string_lossy().as_ref(),
                chart_ref.as_str(),
            ])
            .current_dir(&root)
            .output()
            .map_err(|err| format!("failed to run helm push: {err}"))?;
        stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            errors.push(format!(
                "helm push failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    } else {
        errors.push(
            "release ops push requires --allow-subprocess and --allow-network (or --dry-run)"
                .to_string(),
        );
    }
    let digest = sha256_file(&chart_pkg)?;
    let workspace_ver = workspace_version(&root).unwrap_or_else(|_| "0.1.0".to_string());
    let chart_version = chart_pkg
        .file_name()
        .and_then(|v| v.to_str())
        .and_then(|name| name.strip_prefix("bijux-atlas-"))
        .and_then(|name| name.strip_suffix(".tgz"))
        .unwrap_or(workspace_ver.as_str())
        .to_string();
    let digest_report_path = root.join("ops/release/ops-chart-digest.json");
    let release_manifest_path = root.join("ops/release/ops-release-manifest.json");
    let release_bundle_manifest_path = root.join("ops/release/ops-release-bundle-manifest.json");
    if args.common.allow_write {
        write_json(
            &digest_report_path,
            &serde_json::json!({
                "schema_version": 1,
                "chart_reference": chart_ref,
                "package_path": repo_rel(&root, &chart_pkg),
                "sha256": digest
            }),
        )?;
        write_json(
            &release_manifest_path,
            &serde_json::json!({
                "schema_version": 1,
                "kind": "ops_release_manifest",
                "chart_reference": chart_ref,
                "chart_package_path": repo_rel(&root, &chart_pkg),
                "chart_sha256": digest,
                "chart_version": chart_version,
                "workspace_version": workspace_ver,
                "build_metadata": {
                    "os": std::env::consts::OS,
                    "architecture": std::env::consts::ARCH,
                    "cargo_profile": env_var_text("PROFILE").unwrap_or_else(|| "release".to_string()),
                    "toolchain_versions": collect_toolchain_versions(&root)
                }
            }),
        )?;
        write_json(
            &release_bundle_manifest_path,
            &serde_json::json!({
                "schema_version": 1,
                "kind": "ops_release_bundle_manifest",
                "workspace_version": workspace_ver,
                "chart": {
                    "name": "bijux-atlas",
                    "reference": chart_ref,
                    "version": chart_version,
                    "sha256": digest
                },
                "crates": {
                    "workspace": workspace_ver,
                    "ops_control_plane": "bijux-dev-atlas"
                },
                "source_files": {
                    "ops_release_spec": "ops/release/ops-release.toml",
                    "workspace_manifest": "Cargo.toml"
                }
            }),
        )?;
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_push",
        "status": status,
        "chart_reference": chart_ref,
        "package_path": repo_rel(&root, &chart_pkg),
        "sha256": digest,
        "digest_report_path": repo_rel(&root, &digest_report_path),
        "release_manifest_path": repo_rel(&root, &release_manifest_path),
        "release_bundle_manifest_path": repo_rel(&root, &release_bundle_manifest_path),
        "stdout": stdout,
        "errors": errors
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_digest_verify(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_ops_release_spec(&root)?;
    let out_rel = spec
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("output_dir"))
        .and_then(toml::Value::as_str)
        .unwrap_or("artifacts/release/ops/package");
    let out_dir = root.join(out_rel);
    let chart_pkg = find_ops_chart_tgz(&out_dir)?;
    let digest_report_path = root.join("ops/release/ops-chart-digest.json");
    let mut errors = Vec::<String>::new();
    let mut actual_sha = None;
    if let Some(path) = chart_pkg.as_ref() {
        actual_sha = Some(sha256_file(path)?);
    } else {
        errors.push(format!("no chart package found in {}", out_dir.display()));
    }
    let recorded_sha = if digest_report_path.exists() {
        read_json(&digest_report_path)?
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
    } else {
        errors.push("missing ops/release/ops-chart-digest.json".to_string());
        None
    };
    if actual_sha.is_some() && recorded_sha.is_some() && actual_sha != recorded_sha {
        errors.push("chart digest does not match recorded push digest".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_digest_verify",
        "status": status,
        "actual_sha256": actual_sha,
        "recorded_sha256": recorded_sha,
        "digest_report_path": repo_rel(&root, &digest_report_path),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_pull_test(args: ReleaseOpsPullTestArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.common.repo_root.clone())?;
    let spec = read_ops_release_spec(&root)?;
    let chart_ref = spec
        .get("distribution")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("chart_reference"))
        .and_then(toml::Value::as_str)
        .unwrap_or("oci://ghcr.io/bijux/charts/bijux-atlas")
        .to_string();
    let mut errors = Vec::<String>::new();
    let mut stdout = String::new();
    if args.common.allow_subprocess && args.allow_network {
        let output = ProcessCommand::new("helm")
            .args([
                "pull",
                chart_ref.as_str(),
                "--destination",
                "artifacts/release/ops/pull-test",
            ])
            .current_dir(&root)
            .output()
            .map_err(|err| format!("failed to run helm pull: {err}"))?;
        stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            errors.push(format!(
                "helm pull failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    } else {
        errors.push(
            "release ops pull-test requires --allow-subprocess and --allow-network".to_string(),
        );
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_pull_test",
        "status": status,
        "chart_reference": chart_ref,
        "stdout": stdout,
        "errors": errors
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_bundle_build(args: ReleaseOpsBundleArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.common.repo_root.clone())?;
    if !args.common.allow_write {
        return Err("release ops bundle-build requires --allow-write".to_string());
    }
    let spec = read_ops_release_spec(&root)?;
    let chart_path = root.join(
        spec.get("chart")
            .and_then(toml::Value::as_table)
            .and_then(|v| v.get("path"))
            .and_then(toml::Value::as_str)
            .unwrap_or("ops/k8s/charts/bijux-atlas"),
    );
    let chart_version = release_version_from_chart(&chart_path.join("Chart.yaml"))?;
    let version = args.version.unwrap_or(chart_version);
    let package_out = root.join(
        spec.get("package")
            .and_then(toml::Value::as_table)
            .and_then(|v| v.get("output_dir"))
            .and_then(toml::Value::as_str)
            .unwrap_or("artifacts/release/ops/package"),
    );
    let chart_pkg = find_ops_chart_tgz(&package_out)?;
    let chart_pkg = chart_pkg.ok_or_else(|| {
        format!(
            "chart package is missing in {}; run `release ops package` first",
            package_out.display()
        )
    })?;
    let bundle_dir = root
        .join("artifacts/release/ops/bundle")
        .join(format!("v{version}"));
    fs::create_dir_all(&bundle_dir)
        .map_err(|err| format!("failed to create {}: {err}", bundle_dir.display()))?;
    let bundle_staging = bundle_dir.join("contents");
    fs::create_dir_all(&bundle_staging)
        .map_err(|err| format!("failed to create {}: {err}", bundle_staging.display()))?;
    let include_paths = vec![
        chart_pkg.clone(),
        root.join("ops/k8s/values"),
        root.join("ops/k8s/charts/bijux-atlas/values.schema.json"),
        root.join("ops/inventory"),
        root.join("ops/schema/report/ops-evidence-bundle.schema.json"),
        root.join("ops/k8s/tests/goldens/render-kind.summary.json"),
        root.join("ops/report/generated/ops-values-coverage.json"),
        root.join("ops/report/generated/ops-artifact-lineage.json"),
        root.join("ops/report/generated/release-evidence-bundle.json"),
        root.join("ops/report/generated/readiness-score.json"),
    ];
    let mut copied = Vec::<serde_json::Value>::new();
    for path in include_paths {
        if !path.exists() {
            continue;
        }
        let rel = repo_rel(&root, &path);
        let target = bundle_staging.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        if path.is_dir() {
            let _ = ProcessCommand::new("cp")
                .args([
                    "-R",
                    path.to_string_lossy().as_ref(),
                    target.to_string_lossy().as_ref(),
                ])
                .output();
        } else {
            fs::copy(&path, &target)
                .map_err(|err| format!("failed to copy {}: {err}", path.display()))?;
            copied.push(serde_json::json!({
                "path": rel,
                "sha256": sha256_file(&target)?,
            }));
        }
    }
    copied.sort_by(|a, b| {
        a["path"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["path"].as_str().unwrap_or_default())
    });
    let checksums_path = bundle_staging.join("checksums.json");
    write_json(
        &checksums_path,
        &serde_json::json!({
            "schema_version": 1,
            "artifacts": copied
        }),
    )?;
    let compatibility_path = bundle_staging.join("compatibility-notes.md");
    fs::write(
        &compatibility_path,
        "Ops compatibility notes\n- chart: release-version\n- profiles bundle: release-version\n",
    )
    .map_err(|err| format!("failed to write {}: {err}", compatibility_path.display()))?;
    let tool_versions_path = bundle_staging.join("tool-version-requirements.json");
    write_json(
        &tool_versions_path,
        &serde_json::json!({
            "schema_version": 1,
            "helm": ">=3.14.0",
            "kubectl": ">=1.31.0"
        }),
    )?;
    let tarball = bundle_dir.join(format!("ops-bundle-v{version}.tar.gz"));
    let output = ProcessCommand::new("tar")
        .args([
            "-czf",
            tarball.to_string_lossy().as_ref(),
            "-C",
            bundle_staging.to_string_lossy().as_ref(),
            ".",
        ])
        .output()
        .map_err(|err| format!("failed to build bundle tarball: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to build bundle tarball: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_bundle_build",
        "status": "ok",
        "version": version,
        "bundle_tarball": repo_rel(&root, &tarball),
        "bundle_sha256": sha256_file(&tarball)?,
        "staging_dir": repo_rel(&root, &bundle_staging)
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, 0))
}

fn run_release_ops_bundle_verify(args: ReleaseOpsBundleArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.common.repo_root.clone())?;
    let version = args.version.unwrap_or_else(|| "0.1.0".to_string());
    let bundle_dir = root
        .join("artifacts/release/ops/bundle")
        .join(format!("v{version}"));
    let tarball = bundle_dir.join(format!("ops-bundle-v{version}.tar.gz"));
    let mut errors = Vec::<String>::new();
    if !tarball.exists() {
        errors.push(format!("bundle tarball missing: {}", tarball.display()));
    }
    let staging = bundle_dir.join("contents");
    for required in [
        "checksums.json",
        "tool-version-requirements.json",
        "compatibility-notes.md",
        "ops/schema/report/ops-evidence-bundle.schema.json",
    ] {
        if !staging.join(required).exists() {
            errors.push(format!("bundle missing required file: {required}"));
        }
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_bundle_verify",
        "status": status,
        "version": version,
        "bundle_tarball": repo_rel(&root, &tarball),
        "errors": errors
    });
    let rendered = emit_payload(args.common.format, args.common.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_values_coverage(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let schema_path = root.join("ops/k8s/charts/bijux-atlas/values.schema.json");
    let map_path = root.join("ops/k8s/values/documentation-map.json");
    let schema = read_json(&schema_path)?;
    let map = read_json(&map_path)?;
    let schema_keys = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .map(|rows| rows.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    let mapped = map
        .get("keys")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut covered = BTreeSet::new();
    let mut missing = Vec::<String>::new();
    let mut rows = Vec::<serde_json::Value>::new();
    for row in mapped {
        let Some(key) = row.get("key").and_then(serde_json::Value::as_str) else {
            continue;
        };
        covered.insert(key.to_string());
        let has_docs = row
            .get("docs")
            .and_then(serde_json::Value::as_str)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let internal = row
            .get("internal_only")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if schema_keys.contains(key) && !has_docs && !internal {
            missing.push(format!(
                "values key `{key}` must reference docs or set internal_only=true"
            ));
        }
        rows.push(row);
    }
    for key in &schema_keys {
        if !covered.contains(key) {
            missing.push(format!(
                "values key `{key}` is missing from ops/k8s/values/documentation-map.json"
            ));
        }
    }
    rows.sort_by(|a, b| {
        a.get("key")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .cmp(
                b.get("key")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default(),
            )
    });
    let status = if missing.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_values_coverage",
        "status": status,
        "schema_path": repo_rel(&root, &schema_path),
        "documentation_map_path": repo_rel(&root, &map_path),
        "total_values_keys": schema_keys.len(),
        "covered_values_keys": covered.len(),
        "coverage_percent": if schema_keys.is_empty() { 0.0 } else { (covered.len() as f64 / schema_keys.len() as f64) * 100.0 },
        "rows": rows,
        "errors": missing
    });
    if args.allow_write {
        write_json(
            &root.join("ops/report/generated/ops-values-coverage.json"),
            &payload,
        )?;
    }
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_profiles_verify(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let profiles_path = root.join("ops/k8s/values/profiles.json");
    let profiles = read_json(&profiles_path)?;
    let rows = profiles
        .get("profiles")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::<String>::new();
    let mut output_rows = Vec::<serde_json::Value>::new();
    for row in rows {
        let id = row
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let owner = row
            .get("owner")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let purpose = row
            .get("purpose")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let env = row
            .get("intendedUse")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let risk = row
            .get("risk_level")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if owner.trim().is_empty() {
            errors.push(format!("profile `{id}` must define owner"));
        }
        if purpose.trim().is_empty() {
            errors.push(format!("profile `{id}` must define purpose"));
        }
        if env.trim().is_empty() {
            errors.push(format!("profile `{id}` must define intendedUse"));
        }
        if risk.trim().is_empty() {
            errors.push(format!("profile `{id}` must define risk_level"));
        }
        output_rows.push(serde_json::json!({
            "id": id,
            "owner": owner,
            "intent": purpose,
            "supported_environment": env,
            "risk_tier": risk
        }));
    }
    output_rows.sort_by(|a, b| {
        a["id"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["id"].as_str().unwrap_or_default())
    });
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_profiles_verify",
        "status": status,
        "profiles_path": repo_rel(&root, &profiles_path),
        "profiles": output_rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_lineage_generate(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let chart_digest_path = root.join("ops/release/ops-chart-digest.json");
    let chart_digest_payload = read_json(&chart_digest_path)?;
    let chart_digest = chart_digest_payload
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let image_digest_path = root.join("artifacts/docker-publish/image-digests.json");
    let mut image_digest_source = repo_rel(&root, &image_digest_path);
    let image_digest = if image_digest_path.exists() {
        read_json(&image_digest_path)?
            .get("images")
            .and_then(serde_json::Value::as_array)
            .and_then(|rows| rows.first())
            .and_then(|row| row.get("digest"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string()
    } else {
        let fallback =
            root.join("ops/release/evidence/ops-distribution/install-chart-from-oci/evidence.json");
        if fallback.exists() {
            image_digest_source = repo_rel(&root, &fallback);
            read_json(&fallback)?
                .get("image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string()
        } else {
            String::new()
        }
    };
    let schema_version =
        read_json(&root.join("ops/schema/report/ops-evidence-bundle.schema.json"))?
            .get("schema_version")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(1);
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_artifact_lineage",
        "status": if chart_digest.is_empty() || image_digest.is_empty() { "failed" } else { "ok" },
        "lineage": {
            "chart_digest_sha256": chart_digest,
            "image_digest": image_digest,
            "ops_evidence_schema_version": schema_version
        },
        "sources": {
            "chart_digest_path": repo_rel(&root, &chart_digest_path),
            "image_digest_path": image_digest_source,
            "ops_evidence_schema_path": "ops/schema/report/ops-evidence-bundle.schema.json"
        },
        "errors": if chart_digest.is_empty() || image_digest.is_empty() {
            serde_json::json!(["lineage report requires both chart and image digest"])
        } else {
            serde_json::json!([])
        }
    });
    if args.allow_write {
        write_json(
            &root.join("ops/report/generated/ops-artifact-lineage.json"),
            &payload,
        )?;
    }
    let status = payload
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("failed");
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_provenance_verify(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let doc_path = root.join("docs/06-development/release-and-versioning.md");
    let manifest_path = root.join("ops/release/ops-release-manifest.json");
    let digest_path = root.join("ops/release/ops-chart-digest.json");
    let mut errors = Vec::<String>::new();
    if !doc_path.exists() {
        errors.push("missing docs/06-development/release-and-versioning.md".to_string());
    }
    if !manifest_path.exists() {
        errors.push("missing ops/release/ops-release-manifest.json".to_string());
    }
    if !digest_path.exists() {
        errors.push("missing ops/release/ops-chart-digest.json".to_string());
    }
    if errors.is_empty() {
        let doc = fs::read_to_string(&doc_path)
            .map_err(|err| format!("failed to read {}: {err}", doc_path.display()))?;
        for required_ref in [
            "ops/release/ops-release-manifest.json",
            "ops/release/ops-chart-digest.json",
        ] {
            if !doc.contains(required_ref) {
                errors.push(format!(
                    "ops provenance document must reference `{required_ref}`"
                ));
            }
        }
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_provenance_verify",
        "status": status,
        "doc_path": repo_rel(&root, &doc_path),
        "manifest_path": repo_rel(&root, &manifest_path),
        "chart_digest_path": repo_rel(&root, &digest_path),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_scenario_evidence_verify(
    args: ReleaseOpsPackageArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let scenarios = [
        "install-chart-from-oci",
        "upgrade-chart-from-oci",
        "rollback-chart-from-oci",
        "offline-install-from-bundle",
    ];
    let required_keys = [
        "cluster",
        "helm_version",
        "chart_digest",
        "image_digest",
        "profile",
    ];
    let mut errors = Vec::<String>::new();
    let mut rows = Vec::<serde_json::Value>::new();
    for scenario in scenarios {
        let evidence_path = root
            .join("ops/release/evidence/ops-distribution")
            .join(scenario)
            .join("evidence.json");
        if !evidence_path.exists() {
            errors.push(format!(
                "missing scenario evidence: {}",
                repo_rel(&root, &evidence_path)
            ));
            continue;
        }
        let evidence = read_json(&evidence_path)?;
        let mut missing = Vec::<String>::new();
        for key in required_keys {
            if evidence.get(key).is_none() {
                missing.push(key.to_string());
            }
        }
        if !missing.is_empty() {
            errors.push(format!(
                "{} missing required evidence fields: {}",
                repo_rel(&root, &evidence_path),
                missing.join(", ")
            ));
        }
        rows.push(serde_json::json!({
            "scenario": scenario,
            "evidence_path": repo_rel(&root, &evidence_path),
            "status": if missing.is_empty() { "ok" } else { "failed" },
            "missing_fields": missing
        }));
    }
    rows.sort_by(|a, b| {
        a["scenario"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["scenario"].as_str().unwrap_or_default())
    });
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_scenario_evidence_verify",
        "status": status,
        "rows": rows,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_readiness_summary(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let checks = [
        (
            "validate_package",
            run_release_ops_validate_package(ReleaseOpsPackageArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
                allow_write: false,
                allow_subprocess: false,
            })
            .map(|(_, code)| code == 0)
            .unwrap_or(false),
        ),
        (
            "digest_verify",
            run_release_ops_digest_verify(ReleaseOpsPackageArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
                allow_write: false,
                allow_subprocess: false,
            })
            .map(|(_, code)| code == 0)
            .unwrap_or(false),
        ),
        (
            "values_coverage",
            run_release_ops_values_coverage(ReleaseOpsPackageArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
                allow_write: false,
                allow_subprocess: false,
            })
            .map(|(_, code)| code == 0)
            .unwrap_or(false),
        ),
        (
            "profiles_verify",
            run_release_ops_profiles_verify(ReleaseOpsPackageArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
                allow_write: false,
                allow_subprocess: false,
            })
            .map(|(_, code)| code == 0)
            .unwrap_or(false),
        ),
        (
            "lineage_generate",
            run_release_ops_lineage_generate(ReleaseOpsPackageArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
                allow_write: false,
                allow_subprocess: false,
            })
            .map(|(_, code)| code == 0)
            .unwrap_or(false),
        ),
        (
            "provenance_verify",
            run_release_ops_provenance_verify(ReleaseOpsPackageArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
                allow_write: false,
                allow_subprocess: false,
            })
            .map(|(_, code)| code == 0)
            .unwrap_or(false),
        ),
        (
            "scenario_evidence_verify",
            run_release_ops_scenario_evidence_verify(ReleaseOpsPackageArgs {
                repo_root: Some(root.clone()),
                format: FormatArg::Json,
                out: None,
                allow_write: false,
                allow_subprocess: false,
            })
            .map(|(_, code)| code == 0)
            .unwrap_or(false),
        ),
    ];
    let rows = checks
        .iter()
        .map(|(name, ok)| serde_json::json!({"check": name, "status": if *ok { "ok" } else { "failed" }}))
        .collect::<Vec<_>>();
    let passed = checks.iter().filter(|(_, ok)| *ok).count();
    let total = checks.len();
    let status = if passed == total { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_readiness_summary",
        "status": status,
        "passed": passed,
        "total": total,
        "rows": rows
    });
    if args.allow_write {
        let json_path = root.join("ops/report/generated/ops-release-readiness-summary.json");
        write_json(&json_path, &payload)?;
        let mut md = String::from("# Ops release readiness summary\n\n");
        md.push_str(&format!("- status: `{status}`\n"));
        md.push_str(&format!("- passed checks: `{passed}/{total}`\n\n"));
        md.push_str("| check | status |\n| --- | --- |\n");
        for row in payload
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let check = row
                .get("check")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let row_status = row
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            md.push_str(&format!("| `{check}` | `{row_status}` |\n"));
        }
        let md_path = root.join("ops/report/generated/ops-release-readiness-summary.md");
        fs::write(&md_path, md)
            .map_err(|err| format!("failed to write {}: {err}", md_path.display()))?;
    }
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_ops_publish_plan(args: ReleaseOpsPackageArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_ops_release_spec(&root)?;
    let chart_reference = spec
        .get("distribution")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("chart_reference"))
        .and_then(toml::Value::as_str)
        .unwrap_or("oci://ghcr.io/bijux/charts/bijux-atlas")
        .to_string();
    let bundle_output = spec
        .get("bundle")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("output_dir"))
        .and_then(toml::Value::as_str)
        .unwrap_or("artifacts/release/ops/bundle")
        .to_string();
    let mut artifacts = spec
        .get("artifacts")
        .and_then(toml::Value::as_table)
        .and_then(|v| v.get("include"))
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    artifacts.sort();
    let rows = artifacts
        .iter()
        .map(|path| {
            serde_json::json!({
                "artifact": path,
                "destination": "ops-bundle"
            })
        })
        .chain([
            serde_json::json!({"artifact":"chart package (.tgz)","destination": chart_reference}),
            serde_json::json!({"artifact":"ops bundle tarball","destination": bundle_output}),
        ])
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_publish_plan",
        "status": "ok",
        "chart_reference": chart_reference,
        "bundle_output_dir": bundle_output,
        "artifacts": rows
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_release_ops_compatibility_matrix(
    args: ReleaseOpsPackageArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let workspace_ver = workspace_version(&root).unwrap_or_else(|_| "0.1.0".to_string());
    let ops_manifest_path = root.join("ops/release/ops-release-manifest.json");
    let bundle_manifest_path = root.join("ops/release/ops-release-bundle-manifest.json");
    let chart_ref_default = "oci://ghcr.io/bijux/charts/bijux-atlas".to_string();

    let ops_manifest: serde_json::Value = fs::read_to_string(&ops_manifest_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let bundle_manifest: serde_json::Value = fs::read_to_string(&bundle_manifest_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let chart_version = ops_manifest
        .get("chart_version")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            bundle_manifest
                .get("chart")
                .and_then(|v| v.get("version"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or(workspace_ver.as_str())
        .to_string();
    let chart_reference = ops_manifest
        .get("chart_reference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(chart_ref_default.as_str())
        .to_string();
    let client_version = workspace_ver.clone();

    let rows = vec![serde_json::json!({
        "runtime_version": workspace_ver,
        "chart_version": chart_version,
        "client_version": client_version,
        "chart_reference": chart_reference
    })];
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_ops_compatibility_matrix",
        "status": "ok",
        "rows": rows
    });

    if args.allow_write {
        let md_path = root.join("artifacts/docs/generated/ops-compatibility-matrix.md");
        let json_path = root.join("artifacts/docs/generated/ops-compatibility-matrix.json");
        if let Some(parent) = md_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let markdown = format!(
            "<!-- autogenerated: bijux-dev-atlas docs generate -->\n<!-- do not edit by hand -->\n<!-- Generated by: bijux-dev-atlas docs generate -->\n<!-- Do not edit by hand: regenerate with bijux-dev-atlas docs generate -->\n\n# Ops Compatibility Matrix\n\n| Runtime version | Chart version | Client version | Chart reference |\n| --- | --- | --- | --- |\n| `{}` | `{}` | `{}` | `{}` |\n",
            payload["rows"][0]["runtime_version"].as_str().unwrap_or_default(),
            payload["rows"][0]["chart_version"].as_str().unwrap_or_default(),
            payload["rows"][0]["client_version"].as_str().unwrap_or_default(),
            payload["rows"][0]["chart_reference"].as_str().unwrap_or_default()
        );
        fs::write(&md_path, markdown)
            .map_err(|err| format!("failed to write {}: {err}", md_path.display()))?;
        write_json(&json_path, &payload)?;
    }

    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_release_checksums_generate(args: ReleaseCheckArgs) -> Result<(String, i32), String> {
    let result = run_release_sign(ReleaseSignArgs {
        repo_root: args.repo_root,
        evidence: std::path::PathBuf::from("ops/release/evidence"),
        format: FormatArg::Json,
        out: None,
    })?;
    let payload: serde_json::Value = serde_json::from_str(&result.0).unwrap_or_else(|_| {
        serde_json::json!({"schema_version":1,"kind":"release_checksums_generate","status":"failed","errors":["failed to parse release sign output"]})
    });
    let checksums = payload
        .get("checksums_path")
        .cloned()
        .unwrap_or(serde_json::json!("ops/release/signing/checksums.json"));
    let status = payload
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("failed");
    let wrapped = serde_json::json!({
        "schema_version": 1,
        "kind": "release_checksums_generate",
        "status": status,
        "checksums_path": checksums
    });
    let rendered = emit_payload(args.format, args.out, &wrapped)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_checksums_verify(args: ReleaseCheckArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let checksums_path = root.join("ops/release/signing/checksums.json");
    let checksums = read_json(&checksums_path)?;
    let rows = checksums
        .get("checksums")
        .or_else(|| checksums.get("items"))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::<String>::new();
    let mut verified = 0usize;
    for row in rows {
        let Some(path) = row.get("path").and_then(serde_json::Value::as_str) else {
            errors.push("checksums row missing path".to_string());
            continue;
        };
        let Some(expected) = row.get("sha256").and_then(serde_json::Value::as_str) else {
            errors.push(format!("checksums row missing sha256 for {path}"));
            continue;
        };
        let abs = root.join(path);
        if !abs.exists() {
            errors.push(format!("checksums target missing: {path}"));
            continue;
        }
        let actual = sha256_file(&abs)?;
        if actual != expected {
            errors.push(format!("checksum mismatch: {path}"));
            continue;
        }
        verified += 1;
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let wrapped = serde_json::json!({
        "schema_version": 1,
        "kind": "release_checksums_verify",
        "status": status,
        "checksums_path": repo_rel(&root, &checksums_path),
        "verified_entries": verified,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &wrapped)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn release_spec_allow_deny(spec: &toml::Value) -> (Vec<String>, Vec<String>) {
    let allow = spec
        .get("publish")
        .and_then(toml::Value::as_table)
        .and_then(|publish| publish.get("allow"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let deny = spec
        .get("publish")
        .and_then(toml::Value::as_table)
        .and_then(|publish| publish.get("deny"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    (allow, deny)
}

fn crate_manifest_table(
    root: &Path,
    crate_name: &str,
) -> Result<toml::map::Map<String, toml::Value>, String> {
    let path = root.join("crates").join(crate_name).join("Cargo.toml");
    let value: toml::Value = toml::from_str(
        &fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    value
        .get("package")
        .and_then(toml::Value::as_table)
        .cloned()
        .ok_or_else(|| format!("{} missing [package] table", path.display()))
}

fn run_release_crates_list(args: ReleaseCratesListArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_crates_release_spec(&root)?;
    let (mut publishable, mut blocked) = release_spec_allow_deny(&spec);
    publishable.sort();
    blocked.sort();
    let roles = spec
        .get("roles")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let crate_rows = publishable
        .iter()
        .map(|name| {
            let role = roles
                .get(name)
                .and_then(toml::Value::as_str)
                .unwrap_or("unspecified");
            serde_json::json!({
                "name": name,
                "publish": true,
                "role": role
            })
        })
        .chain(blocked.iter().map(|name| {
            let role = roles
                .get(name)
                .and_then(toml::Value::as_str)
                .unwrap_or("unspecified");
            serde_json::json!({
                "name": name,
                "publish": false,
                "role": role
            })
        }))
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_crates_list",
        "release_line": release_line_from_spec(&spec, &root),
        "versioning_model": spec.get("versioning_model").and_then(toml::Value::as_str).unwrap_or("workspace-unified"),
        "crates": crate_rows
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_release_crates_validate_metadata(
    args: ReleaseCratesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let workspace_manifest: toml::Value = toml::from_str(
        &fs::read_to_string(root.join("Cargo.toml"))
            .map_err(|err| format!("failed to read root Cargo.toml: {err}"))?,
    )
    .map_err(|err| format!("failed to parse root Cargo.toml: {err}"))?;
    let workspace_package = workspace_manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table);
    let spec = read_crates_release_spec(&root)?;
    let (publishable, _) = release_spec_allow_deny(&spec);
    let required_fields = spec
        .get("metadata_requirements")
        .and_then(toml::Value::as_table)
        .and_then(|metadata| metadata.get("required_package_fields"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let mut errors = Vec::<String>::new();
    let mut checked = Vec::<String>::new();
    for crate_name in publishable {
        let package = crate_manifest_table(&root, &crate_name)?;
        for key in &required_fields {
            let value = package.get(key);
            let missing = match value {
                None => true,
                Some(toml::Value::String(text)) => text.trim().is_empty(),
                Some(toml::Value::Array(values)) => values.is_empty(),
                Some(toml::Value::Table(table)) => !table
                    .get("workspace")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
                _ => false,
            };
            if missing {
                errors.push(format!("crate `{crate_name}` missing package.{key}"));
            }
        }
        let readme_rel = package
            .get("readme")
            .and_then(toml::Value::as_str)
            .unwrap_or("README.md");
        let readme_path = root.join("crates").join(&crate_name).join(readme_rel);
        if !readme_path.exists() {
            errors.push(format!(
                "crate `{crate_name}` readme path does not exist: {}",
                readme_path.display()
            ));
        } else {
            let readme = fs::read_to_string(&readme_path)
                .map_err(|err| format!("failed to read {}: {err}", readme_path.display()))?;
            errors.extend(validate_publishable_package_surface(
                &crate_name,
                &package,
                workspace_package,
                &readme,
            ));
        }
        checked.push(crate_name);
    }
    checked.sort();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_crates_validate_metadata",
        "status": status,
        "checked_crates": checked,
        "required_fields": required_fields,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn resolve_package_field(
    package: &toml::value::Table,
    workspace_package: Option<&toml::value::Table>,
    key: &str,
) -> Option<String> {
    let value = package.get(key)?;
    match value {
        toml::Value::String(text) => Some(text.to_string()),
        toml::Value::Table(table) => table
            .get("workspace")
            .and_then(toml::Value::as_bool)
            .filter(|workspace| *workspace)
            .and(workspace_package)
            .and_then(|workspace| workspace.get(key))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn validate_publishable_package_surface(
    crate_name: &str,
    package: &toml::value::Table,
    workspace_package: Option<&toml::value::Table>,
    readme: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    let repository = resolve_package_field(package, workspace_package, "repository");
    let homepage = resolve_package_field(package, workspace_package, "homepage");
    let documentation = resolve_package_field(package, workspace_package, "documentation");
    for (field_name, value) in [
        ("repository", repository.as_deref()),
        ("homepage", homepage.as_deref()),
        ("documentation", documentation.as_deref()),
    ] {
        let Some(url) = value else {
            continue;
        };
        if !url.starts_with("https://") {
            errors.push(format!(
                "crate `{crate_name}` package.{field_name} must be an absolute https URL"
            ));
        }
    }
    if let (Some(repository), Some(homepage)) = (repository.as_deref(), homepage.as_deref()) {
        if repository == homepage {
            errors.push(format!(
                "crate `{crate_name}` package.homepage must not duplicate package.repository"
            ));
        }
    }
    if let (Some(homepage), Some(documentation)) = (homepage.as_deref(), documentation.as_deref()) {
        if homepage == documentation {
            errors.push(format!(
                "crate `{crate_name}` package.homepage and package.documentation must point to distinct surfaces"
            ));
        }
    }
    if let Some(homepage) = homepage.as_deref() {
        if !readme.contains(homepage) {
            errors.push(format!(
                "crate `{crate_name}` README must link to package.homepage"
            ));
        }
    }
    if let Some(documentation) = documentation.as_deref() {
        if !readme.contains(documentation) {
            errors.push(format!(
                "crate `{crate_name}` README must link to package.documentation"
            ));
        }
    }
    for target in markdown_link_targets(readme) {
        if target.starts_with("../") || target.contains("/../") {
            errors.push(format!(
                "crate `{crate_name}` README contains parent-relative link `{target}` that will not be publish-safe"
            ));
        }
    }
    errors
}

fn markdown_link_targets(markdown: &str) -> Vec<&str> {
    let mut targets = Vec::new();
    let mut cursor = 0usize;
    while let Some(open_bracket) = markdown[cursor..].find('[') {
        let open_bracket = cursor + open_bracket;
        let Some(close_bracket) = markdown[open_bracket..].find(']') else {
            break;
        };
        let close_bracket = open_bracket + close_bracket;
        let Some(open_paren) = markdown[close_bracket..].find('(') else {
            cursor = close_bracket + 1;
            continue;
        };
        let open_paren = close_bracket + open_paren;
        if open_paren != close_bracket + 1 {
            cursor = close_bracket + 1;
            continue;
        }
        let Some(close_paren) = markdown[open_paren + 1..].find(')') else {
            break;
        };
        let close_paren = open_paren + 1 + close_paren;
        let target = markdown[open_paren + 1..close_paren].trim();
        if !target.is_empty() {
            targets.push(target);
        }
        cursor = close_paren + 1;
    }
    targets
}

fn run_release_crates_validate_publish_flags(
    args: ReleaseCratesValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_crates_release_spec(&root)?;
    let (publishable, blocked) = release_spec_allow_deny(&spec);
    let mut errors = Vec::<String>::new();
    let mut checked = Vec::<String>::new();
    for crate_name in publishable {
        let package = crate_manifest_table(&root, &crate_name)?;
        if !package
            .get("publish")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true)
        {
            errors.push(format!(
                "crate `{crate_name}` is publishable but has package.publish = false"
            ));
        }
        checked.push(crate_name);
    }
    for crate_name in blocked {
        let package = crate_manifest_table(&root, &crate_name)?;
        if package
            .get("publish")
            .and_then(toml::Value::as_bool)
            .unwrap_or(true)
        {
            errors.push(format!(
                "crate `{crate_name}` is blocked but missing package.publish = false"
            ));
        }
        checked.push(crate_name);
    }
    checked.sort();
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_crates_validate_publish_flags",
        "status": status,
        "checked_crates": checked,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn workspace_version(root: &Path) -> Result<String, String> {
    let manifest: toml::Value = toml::from_str(
        &fs::read_to_string(root.join("Cargo.toml"))
            .map_err(|err| format!("failed to read root Cargo.toml: {err}"))?,
    )
    .map_err(|err| format!("failed to parse root Cargo.toml: {err}"))?;
    manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table)
        .and_then(|pkg| pkg.get("version"))
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| "workspace.package.version not found in root Cargo.toml".to_string())
}

fn parse_semver_triplet(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    Some((major, minor, patch))
}

fn collect_api_surface_entries(root: &Path, crate_name: &str) -> Result<Vec<String>, String> {
    let lib = root.join("crates").join(crate_name).join("src/lib.rs");
    let text = fs::read_to_string(&lib)
        .map_err(|err| format!("failed to read {}: {err}", lib.display()))?;
    let re =
        Regex::new(r#"^\s*pub\s+(mod|use|fn|struct|enum|trait|const|type)\s+([A-Za-z0-9_]+)?"#)
            .map_err(|err| format!("failed to build api surface regex: {err}"))?;
    let mut entries = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("//"))
        .filter_map(|line| {
            re.captures(line).map(|caps| {
                let kind = caps.get(1).map(|m| m.as_str()).unwrap_or("item");
                let name = caps.get(2).map(|m| m.as_str()).unwrap_or("_");
                format!("{kind}:{name}")
            })
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries.dedup();
    Ok(entries)
}

fn run_release_api_surface_snapshot(
    args: ReleaseApiSurfaceSnapshotArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_crates_release_spec(&root)?;
    let (allow, _) = release_spec_allow_deny(&spec);
    let selected = if args.all {
        allow
    } else if let Some(crate_name) = args.crate_name.clone() {
        vec![crate_name]
    } else {
        return Err("release api-surface snapshot requires --all or --crate-name".to_string());
    };
    let mut rows = Vec::<serde_json::Value>::new();
    for crate_name in selected {
        let entries = collect_api_surface_entries(&root, &crate_name)?;
        let current_path = root
            .join("ops/release/api-surface/current")
            .join(format!("{crate_name}.json"));
        if let Some(parent) = current_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        write_json(
            &current_path,
            &serde_json::json!({
                "schema_version": 1,
                "crate": crate_name,
                "items": entries
            }),
        )?;
        let mut row = serde_json::json!({
            "crate": crate_name,
            "current_snapshot": repo_rel(&root, &current_path),
        });
        if args.write_golden {
            let golden_path = root
                .join("ops/release/api-surface/golden")
                .join(format!("{crate_name}.json"));
            if let Some(parent) = golden_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
            }
            fs::copy(&current_path, &golden_path).map_err(|err| {
                format!(
                    "failed to copy snapshot {} -> {}: {err}",
                    current_path.display(),
                    golden_path.display()
                )
            })?;
            row["golden_snapshot"] = serde_json::json!(repo_rel(&root, &golden_path));
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| {
        a["crate"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["crate"].as_str().unwrap_or_default())
    });
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_api_surface_snapshot",
        "status": "ok",
        "rows": rows
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_release_semver_check(args: ReleaseSemverCheckArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_json(&root.join("configs/sources/release/semver-api-policy.json"))?;
    let current_version = args.version.unwrap_or(workspace_version(&root)?);
    let (major, _, _) = parse_semver_triplet(&current_version)
        .ok_or_else(|| format!("invalid semver version: {current_version}"))?;
    let baseline_version = policy
        .get("baseline_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("0.1.0")
        .to_string();
    let (baseline_major, _, _) = parse_semver_triplet(&baseline_version)
        .ok_or_else(|| format!("invalid baseline version in policy: {baseline_version}"))?;
    let spec = read_crates_release_spec(&root)?;
    let (publishable, _) = release_spec_allow_deny(&spec);
    let mut errors = Vec::<String>::new();
    let mut crate_reports = Vec::<serde_json::Value>::new();
    for crate_name in publishable {
        let current = collect_api_surface_entries(&root, &crate_name)?;
        let golden_path = root
            .join("ops/release/api-surface/golden")
            .join(format!("{crate_name}.json"));
        if !golden_path.exists() {
            errors.push(format!(
                "missing golden API surface snapshot for crate `{crate_name}`"
            ));
            continue;
        }
        let golden = read_json(&golden_path)?;
        let previous = golden
            .get("items")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        let current_set = current.iter().cloned().collect::<BTreeSet<_>>();
        let removed = previous
            .difference(&current_set)
            .cloned()
            .collect::<Vec<_>>();
        let added = current_set
            .difference(&previous)
            .cloned()
            .collect::<Vec<_>>();
        let requires_semver_bump = !removed.is_empty();
        if requires_semver_bump && major <= baseline_major {
            errors.push(format!(
                "crate `{crate_name}` removed public API items but version `{current_version}` does not increase major above baseline `{baseline_version}`"
            ));
        }
        crate_reports.push(serde_json::json!({
            "crate": crate_name,
            "removed": removed,
            "added": added,
            "requires_semver_bump": requires_semver_bump,
            "semver_rule_evaluated": true
        }));
    }
    crate_reports.sort_by(|a, b| {
        a["crate"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["crate"].as_str().unwrap_or_default())
    });
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_semver_check",
        "status": status,
        "version": current_version,
        "baseline_version": baseline_version,
        "rules": policy.get("rules").cloned().unwrap_or_else(|| serde_json::json!([])),
        "crates": crate_reports,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_msrv_verify(args: ReleaseMsrvVerifyArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let cargo = toml::from_str::<toml::Value>(
        &fs::read_to_string(root.join("Cargo.toml"))
            .map_err(|err| format!("failed to read root Cargo.toml: {err}"))?,
    )
    .map_err(|err| format!("failed to parse root Cargo.toml: {err}"))?;
    let workspace_msrv = cargo
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|w| w.get("package"))
        .and_then(toml::Value::as_table)
        .and_then(|p| p.get("rust-version"))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let toolchain = toml::from_str::<toml::Value>(
        &fs::read_to_string(root.join("rust-toolchain.toml"))
            .map_err(|err| format!("failed to read rust-toolchain.toml: {err}"))?,
    )
    .map_err(|err| format!("failed to parse rust-toolchain.toml: {err}"))?;
    let toolchain_channel = toolchain
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|t| t.get("channel"))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let msrv_doc = fs::read_to_string(root.join("docs/06-development/workspace-and-tooling.md"))
        .map_err(|err| {
            format!("failed to read docs/06-development/workspace-and-tooling.md: {err}")
        })?;
    let mut errors = Vec::<String>::new();
    if workspace_msrv != toolchain_channel {
        errors.push(format!(
            "workspace rust-version `{workspace_msrv}` does not match rust-toolchain channel `{toolchain_channel}`"
        ));
    }
    if !msrv_doc.contains(&workspace_msrv) {
        errors.push(format!(
            "docs/06-development/workspace-and-tooling.md does not mention workspace MSRV `{workspace_msrv}`"
        ));
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_msrv_verify",
        "status": status,
        "workspace_msrv": workspace_msrv,
        "toolchain_channel": toolchain_channel,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_crates_publish_plan(args: ReleaseCratesListArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_crates_release_spec(&root)?;
    let (mut publishable, _) = release_spec_allow_deny(&spec);
    publishable.sort();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_crates_publish_plan",
        "status": "ok",
        "deterministic_ordering": true,
        "release_line": release_line_from_spec(&spec, &root),
        "crates": publishable
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn package_policy(root: &Path) -> Result<serde_json::Value, String> {
    read_json(&root.join("configs/sources/release/crate-package-policy.json"))
}

fn dependency_policy(root: &Path) -> Result<serde_json::Value, String> {
    read_json(&root.join("configs/sources/release/dependency-policy.json"))
}

fn run_release_crates_dry_run(args: ReleaseCratesDryRunArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let spec = read_crates_release_spec(&root)?;
    let policy = package_policy(&root)?;
    let (mut publishable, _) = release_spec_allow_deny(&spec);
    publishable.sort();
    let deny_patterns = policy
        .get("deny_patterns")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    let max_bytes = policy
        .get("size_budget_bytes")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1_500_000);
    let require_changelog = policy
        .get("require_changelog_in_package")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut rows = Vec::<serde_json::Value>::new();
    let mut dependency_audit_rows = Vec::<serde_json::Value>::new();
    let version = workspace_version(&root)?;
    for crate_name in publishable {
        let output = ProcessCommand::new("cargo")
            .args(["package", "-p", &crate_name, "--allow-dirty", "--list"])
            .current_dir(&root)
            .output()
            .map_err(|err| {
                format!("failed to run cargo package --list for `{crate_name}`: {err}")
            })?;
        if !output.status.success() {
            errors.push(format!(
                "cargo package --list failed for `{crate_name}`: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
            continue;
        }
        let members = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let has_license = members
            .iter()
            .any(|m| m.ends_with("LICENSE") || m.ends_with("LICENSE.txt"));
        let has_readme = members.iter().any(|m| m.ends_with("README.md"));
        let has_changelog = members.iter().any(|m| m.ends_with("CHANGELOG.md"));
        if !has_license {
            errors.push(format!("crate `{crate_name}` package list missing LICENSE"));
        }
        if !has_readme {
            errors.push(format!(
                "crate `{crate_name}` package list missing README.md"
            ));
        }
        if require_changelog && !has_changelog {
            errors.push(format!(
                "crate `{crate_name}` package list missing CHANGELOG.md required by policy"
            ));
        }
        for pattern in &deny_patterns {
            if members.iter().any(|m| m.contains(pattern)) {
                errors.push(format!(
                    "crate `{crate_name}` package contains denied pattern `{pattern}`"
                ));
            }
        }
        let prefix = format!("{crate_name}-{version}/");
        let crate_root = root.join("crates").join(&crate_name);
        let mut size_bytes = 0_u64;
        for member in &members {
            let member_rel = member.strip_prefix(&prefix).unwrap_or(member);
            let path = crate_root.join(member_rel);
            if path.exists() {
                size_bytes = size_bytes.saturating_add(
                    fs::metadata(&path)
                        .map_err(|err| format!("failed to stat {}: {err}", path.display()))?
                        .len(),
                );
            }
        }
        if size_bytes > max_bytes {
            let message = format!(
                "crate `{crate_name}` package size {} exceeds budget {} bytes",
                size_bytes, max_bytes
            );
            if args.enforce_size_budget {
                errors.push(message);
            } else {
                warnings.push(message);
            }
        }
        let manifest_raw: toml::Value = toml::from_str(
            &fs::read_to_string(root.join("crates").join(&crate_name).join("Cargo.toml"))
                .map_err(|err| format!("failed to read {crate_name} manifest: {err}"))?,
        )
        .map_err(|err| format!("failed to parse {crate_name} manifest: {err}"))?;
        let deps = manifest_raw
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        let mut network = Vec::<String>::new();
        let mut native = Vec::<String>::new();
        let mut high_risk = Vec::<String>::new();
        for dep_name in deps.keys() {
            match dep_name.as_str() {
                "reqwest" | "redis" | "tokio" | "opentelemetry-otlp" => {
                    network.push(dep_name.clone());
                }
                "rusqlite" | "libsqlite3-sys" | "tikv-jemallocator" => {
                    native.push(dep_name.clone());
                }
                _ => {}
            }
            if dep_name == "reqwest" || dep_name == "redis" {
                high_risk.push(dep_name.clone());
            }
        }
        network.sort();
        native.sort();
        high_risk.sort();
        dependency_audit_rows.push(serde_json::json!({
            "crate": crate_name,
            "direct_dependency_count": deps.len(),
            "risk_categories": {
                "network": network,
                "native": native,
                "high_risk_review": high_risk
            }
        }));
        rows.push(serde_json::json!({
            "crate": crate_name,
            "has_license": has_license,
            "has_readme": has_readme,
            "has_changelog": has_changelog,
            "member_count": members.len(),
            "size_bytes": size_bytes
        }));
    }
    dependency_audit_rows.sort_by(|a, b| {
        a["crate"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["crate"].as_str().unwrap_or_default())
    });
    let audit_path = root.join("artifacts/release/crates/dependency-audit.json");
    write_json(
        &audit_path,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "crate_dependency_audit",
            "rows": dependency_audit_rows
        }),
    )?;
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_crates_dry_run",
        "status": status,
        "size_budget_bytes": max_bytes,
        "dependency_audit_report": repo_rel(&root, &audit_path),
        "rows": rows,
        "warnings": warnings,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_plan(args: ReleasePlanArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_publish_policy(&root)?;
    let publishable = policy["publishable_crates"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let blocked = policy["blocked_crates"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let strategy = policy["versioning_strategy"]
        .as_str()
        .unwrap_or("workspace-unified");
    let payload = serde_json::json!({
        "kind": "release_plan",
        "repo_root": root.display().to_string(),
        "versioning_strategy": strategy,
        "publishable_crates": publishable,
        "blocked_crates": blocked
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn normalize_transition_versions(
    from_version: Option<String>,
    to_version: Option<String>,
) -> (String, String) {
    (
        from_version.unwrap_or_else(|| "0.1.1".to_string()),
        to_version.unwrap_or_else(|| "0.2.0".to_string()),
    )
}

fn run_release_compatibility_check(
    args: ReleaseCompatibilityCheckArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let table_rel = "ops/e2e/scenarios/upgrade/version-compatibility.json";
    let table = read_json(&root.join(table_rel))?;
    let (from_version, to_version) =
        normalize_transition_versions(args.from_version, args.to_version);
    let rows = table
        .get("compatibility")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let matching = rows.into_iter().find(|row| {
        row.get("from").and_then(serde_json::Value::as_str) == Some(from_version.as_str())
            && row.get("to").and_then(serde_json::Value::as_str) == Some(to_version.as_str())
    });
    let mut errors = Vec::<String>::new();
    let status = if let Some(row) = matching {
        if row
            .get("supported")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            "ok"
        } else {
            errors.push("transition exists but is marked unsupported".to_string());
            "failed"
        }
    } else {
        errors.push("transition not found in compatibility table".to_string());
        "failed"
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_compatibility_check",
        "status": status,
        "from_version": from_version,
        "to_version": to_version,
        "compatibility_table": table_rel,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_transition_plan(
    args: ReleaseTransitionPlanArgs,
    mode: &str,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let (from_version, to_version) =
        normalize_transition_versions(args.from_version, args.to_version);
    let scenario_id = if mode == "rollback" {
        "rollback-after-successful-upgrade"
    } else {
        "upgrade-patch"
    };
    let scenario_rel = format!("ops/e2e/scenarios/upgrade/{scenario_id}.json");
    let spec = read_json(&root.join(&scenario_rel))?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": format!("release_{}_plan", mode),
        "status": "ok",
        "from_version": from_version,
        "to_version": to_version,
        "scenario_id": scenario_id,
        "steps": spec.get("steps").cloned().unwrap_or_else(|| serde_json::json!([])),
        "artifacts": {
            "compatibility_table": "ops/e2e/scenarios/upgrade/version-compatibility.json",
            "contracts": "ops/e2e/scenarios/upgrade/contracts.json"
        }
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_release_validate(args: ReleaseValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root.clone())?;
    let policy = read_publish_policy(&root)?;
    let feature_policy = read_json(&root.join("configs/sources/release/feature-policy.json"))?;
    let missing_docs_policy =
        read_json(&root.join("configs/sources/release/missing-docs-policy.json"))?;
    let dependency_policy = dependency_policy(&root)?;
    let feature_doc = fs::read_to_string(root.join("docs/07-reference/feature-flags.md"))
        .map_err(|err| format!("failed to read docs/07-reference/feature-flags.md: {err}"))?;
    let workspace_manifest: toml::Value = toml::from_str(
        &fs::read_to_string(root.join("Cargo.toml"))
            .map_err(|err| format!("failed to read root Cargo.toml: {err}"))?,
    )
    .map_err(|err| format!("failed to parse root Cargo.toml: {err}"))?;
    let workspace_rust_version = workspace_manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table)
        .and_then(|pkg| pkg.get("rust-version"))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let workspace_package = workspace_manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table);
    let publishable = policy["publishable_crates"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut checked_crates = Vec::<String>::new();
    if !root.join("LICENSE").exists() {
        errors.push("missing root LICENSE file".to_string());
    }
    if !root.join("CHANGELOG.md").exists() {
        errors.push("missing CHANGELOG.md".to_string());
    }
    for crate_name in publishable.iter().filter_map(serde_json::Value::as_str) {
        let docs_entry = missing_docs_policy
            .get("crate_policy")
            .and_then(serde_json::Value::as_object)
            .and_then(|entries| entries.get(crate_name))
            .cloned();
        if docs_entry.is_none() {
            errors.push(format!(
                "missing docs policy entry for crate `{crate_name}` in configs/sources/release/missing-docs-policy.json"
            ));
        }
        let manifest_path = root.join("crates").join(crate_name).join("Cargo.toml");
        let readme_path = root.join("crates").join(crate_name).join("README.md");
        let text = fs::read_to_string(&manifest_path)
            .map_err(|err| format!("failed to read {}: {err}", manifest_path.display()))?;
        let manifest: toml::Value = toml::from_str(&text)
            .map_err(|err| format!("failed to parse {}: {err}", manifest_path.display()))?;
        let pkg = manifest.get("package").and_then(toml::Value::as_table);
        let forbidden_licenses = dependency_policy
            .get("forbidden_licenses")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        let package_license = pkg
            .and_then(|table| table.get("license"))
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_string();
        if forbidden_licenses.contains(&package_license) {
            errors.push(format!(
                "crate `{crate_name}` uses forbidden license `{package_license}`"
            ));
        }
        let missing = |key: &str| {
            let Some(value) = pkg.and_then(|v| v.get(key)) else {
                return true;
            };
            match value {
                toml::Value::String(text) => text.trim().is_empty(),
                toml::Value::Table(table) => !table
                    .get("workspace")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
                _ => true,
            }
        };
        for key in ["description", "license", "repository", "documentation"] {
            if missing(key) {
                errors.push(format!("{} missing package.{key}", manifest_path.display()));
            }
        }
        let rust_version_ok = pkg
            .and_then(|v| v.get("rust-version"))
            .is_some_and(|value| match value {
                toml::Value::String(text) => text == &workspace_rust_version,
                toml::Value::Table(table) => table
                    .get("workspace")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
                _ => false,
            });
        if !rust_version_ok {
            errors.push(format!(
                "{} must set package.rust-version to workspace policy `{}`",
                manifest_path.display(),
                workspace_rust_version
            ));
        }
        if !readme_path.exists() {
            errors.push(format!("missing crate README: {}", readme_path.display()));
        } else if let Some(pkg_table) = pkg {
            let readme = fs::read_to_string(&readme_path)
                .map_err(|err| format!("failed to read {}: {err}", readme_path.display()))?;
            errors.extend(validate_publishable_package_surface(
                crate_name,
                pkg_table,
                workspace_package,
                &readme,
            ));
        }
        for section in ["dependencies", "build-dependencies", "dev-dependencies"] {
            let Some(deps) = manifest.get(section).and_then(toml::Value::as_table) else {
                continue;
            };
            for (dep_name, value) in deps {
                if (dep_name == "bijux-atlas-core"
                    || dep_name.starts_with("bijux-atlas-")
                    || dep_name == "bijux-dev-atlas")
                    && value
                        .as_table()
                        .and_then(|table| table.get("path"))
                        .is_some()
                {
                    errors.push(format!(
                        "{} section `{section}` dependency `{dep_name}` uses path, forbidden for publishable crate manifests",
                        manifest_path.display(),
                    ));
                }
                if value
                    .as_table()
                    .and_then(|table| table.get("git"))
                    .is_some()
                {
                    errors.push(format!(
                        "{} section `{section}` dependency `{dep_name}` uses git source, forbidden for publishable crates",
                        manifest_path.display(),
                    ));
                }
            }
        }
        let dev_deps = manifest
            .get("dev-dependencies")
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        let deps = manifest
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        let duplicates = deps
            .iter()
            .filter_map(|(name, dep_value)| {
                if !dev_deps.contains_key(name) {
                    return None;
                }
                let optional = dep_value
                    .as_table()
                    .and_then(|table| table.get("optional"))
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false);
                if optional {
                    None
                } else {
                    Some(name.clone())
                }
            })
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            errors.push(format!(
                "crate `{crate_name}` duplicates dependencies in [dependencies] and [dev-dependencies]: {}",
                duplicates.join(", ")
            ));
        }
        let features = manifest
            .get("features")
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        for feature_name in features.keys() {
            if feature_name == "default" {
                continue;
            }
            if !feature_doc.contains(&format!("`{crate_name}`"))
                || !feature_doc.contains(&format!("`{feature_name}`"))
            {
                errors.push(format!(
                    "crate `{crate_name}` feature `{feature_name}` is not documented in docs/07-reference/feature-flags.md"
                ));
            }
        }
        let allow_empty_features = feature_policy
            .get("allow_empty_features")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        for (feature_name, definition) in &features {
            if feature_name == "default" {
                continue;
            }
            let is_empty = definition.as_array().is_some_and(|items| items.is_empty());
            if is_empty && !allow_empty_features.contains(feature_name) {
                errors.push(format!(
                    "crate `{crate_name}` feature `{feature_name}` has no dependencies and is not allowed by policy"
                ));
            }
        }
        let default_feature_allow = feature_policy
            .get("allowed_default_features")
            .and_then(serde_json::Value::as_object)
            .cloned()
            .unwrap_or_default();
        if let Some(defaults) = features.get("default").and_then(toml::Value::as_array) {
            let allowed = default_feature_allow
                .get(crate_name)
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                .collect::<BTreeSet<_>>();
            for feature in defaults.iter().filter_map(toml::Value::as_str) {
                if !allowed.contains(feature) {
                    errors.push(format!(
                        "crate `{crate_name}` default feature `{feature}` is not allowed by configs/sources/release/feature-policy.json"
                    ));
                }
            }
        }
        let examples_dir = root.join("crates").join(crate_name).join("examples");
        if examples_dir.exists() {
            let has_examples = fs::read_dir(&examples_dir)
                .map_err(|err| format!("failed to read {}: {err}", examples_dir.display()))?
                .filter_map(Result::ok)
                .any(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext == "rs")
                });
            if has_examples {
                let status = ProcessCommand::new("cargo")
                    .args(["check", "-p", crate_name, "--examples", "--locked"])
                    .current_dir(&root)
                    .status()
                    .map_err(|err| {
                        format!("failed to run cargo check examples for {crate_name}: {err}")
                    })?;
                if !status.success() {
                    errors.push(format!(
                        "example compilation failed for crate `{crate_name}`"
                    ));
                }
            }
        }
        checked_crates.push(crate_name.to_string());
    }

    let duplicate_threshold = dependency_policy
        .get("duplicate_dependency_threshold")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(30) as usize;
    let mut duplicates = BTreeSet::<String>::new();
    let mut duplicate_scan_failed = None;
    for crate_name in &checked_crates {
        let duplicate_output = ProcessCommand::new("cargo")
            .args([
                "tree",
                "-d",
                "-p",
                crate_name,
                "--all-features",
                "-e",
                "normal,build",
                "--prefix",
                "none",
            ])
            .current_dir(&root)
            .output();
        match duplicate_output {
            Ok(output) if output.status.success() => {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some(name) = trimmed.split_whitespace().next() {
                        duplicates.insert(name.to_string());
                    }
                }
            }
            Ok(output) => {
                duplicate_scan_failed = Some(format!(
                    "cargo tree duplicate dependency scan failed for {crate_name}: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
                break;
            }
            Err(err) => {
                duplicate_scan_failed = Some(format!(
                    "failed to run cargo tree duplicate scan for {crate_name}: {err}"
                ));
                break;
            }
        }
    }
    if let Some(message) = duplicate_scan_failed {
        warnings.push(message);
    } else if duplicates.len() > duplicate_threshold {
        warnings.push(format!(
            "duplicate dependency count {} exceeds threshold {}",
            duplicates.len(),
            duplicate_threshold
        ));
    }

    let cargo_deny_required = dependency_policy
        .get("cargo_deny")
        .and_then(serde_json::Value::as_object)
        .and_then(|row| row.get("required"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let cargo_deny_enabled = dependency_policy
        .get("cargo_deny")
        .and_then(serde_json::Value::as_object)
        .and_then(|row| row.get("enabled"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if cargo_deny_enabled {
        let cargo_deny_config = root.join("configs/sources/security/deny.toml");
        match ProcessCommand::new("cargo")
            .args([
                "deny",
                "check",
                "--config",
                cargo_deny_config
                    .to_str()
                    .ok_or_else(|| "cargo-deny config path is not valid UTF-8".to_string())?,
                "licenses",
                "bans",
                "sources",
            ])
            .current_dir(&root)
            .output()
        {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let message = format!(
                    "cargo deny check failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
                if cargo_deny_required {
                    errors.push(message);
                } else {
                    warnings.push(message);
                }
            }
            Err(err) => {
                let message = format!("cargo deny is not available: {err}");
                if cargo_deny_required {
                    errors.push(message);
                } else {
                    warnings.push(message);
                }
            }
        }
    }

    let examples_runtime_dir = root.join("configs/examples/runtime");
    if !examples_runtime_dir.exists() {
        errors.push("missing configs/examples/runtime directory".to_string());
    } else {
        let mut has_toml = false;
        for entry in fs::read_dir(&examples_runtime_dir)
            .map_err(|err| format!("failed to read {}: {err}", examples_runtime_dir.display()))?
        {
            let entry =
                entry.map_err(|err| format!("failed to read runtime config entry: {err}"))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
                continue;
            }
            has_toml = true;
            let text = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            let parsed: toml::Value = toml::from_str(&text)
                .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
            let has_server = parsed
                .get("server")
                .and_then(toml::Value::as_table)
                .is_some();
            if !has_server {
                errors.push(format!(
                    "{} missing required [server] table",
                    path.display()
                ));
            }
        }
        if !has_toml {
            errors
                .push("no runtime TOML examples found under configs/examples/runtime".to_string());
        }
    }
    let changelog_validate = run_release_changelog_validate(ReleaseChangelogValidateArgs {
        repo_root: Some(root.clone()),
        version: args.version.clone(),
        tag: None,
        format: FormatArg::Json,
        out: None,
    })?;
    let changelog_payload: serde_json::Value = serde_json::from_str(&changelog_validate.0)
        .map_err(|err| format!("failed to parse changelog validation payload: {err}"))?;
    if changelog_validate.1 != 0 {
        errors.push("release changelog validation failed".to_string());
    }
    let exit = if errors.is_empty() { 0 } else { 1 };
    let payload = serde_json::json!({
        "kind": "release_validate",
        "repo_root": root.display().to_string(),
        "checked_crates": checked_crates,
        "warnings": warnings,
        "errors": errors,
        "changelog_validation": changelog_payload
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, exit))
}

fn repo_rel<'a>(root: &'a Path, path: &'a Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn run_release_sign(args: ReleaseSignArgs) -> Result<(String, i32), String> {
    let repo_root_arg = args.repo_root.clone();
    let root = resolve_repo_root(repo_root_arg.clone())?;
    ensure_json(&root.join("configs/schemas/contracts/release/signing-policy.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/release/checksum-list.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/release/release-sign.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/release/provenance.schema.json"))?;

    let evidence_dir = if args.evidence.is_absolute() {
        args.evidence
    } else {
        root.join(args.evidence)
    };
    let policy_path = root.join("ops/release/signing/policy.yaml");
    let checksums_path = root.join("ops/release/signing/checksums.json");
    let sign_report_path = root.join("ops/release/signing/release-sign.json");
    let provenance_path = root.join("ops/release/provenance.json");
    let manifest_path = evidence_dir.join("manifest.json");
    let identity_path = evidence_dir.join("identity.json");

    let policy = read_yaml(&policy_path)?;
    let identity = read_json(&identity_path)?;
    let mut manifest = read_json(&manifest_path)?;
    let signed_items = policy
        .get("signed_items")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let mut errors = Vec::new();
    let release_id = identity
        .get("release_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unsigned");
    let git_sha = identity
        .get("git_sha")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("0000000000000000000000000000000000000000");
    let governance_version = identity
        .get("governance_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("main@unknown");
    let provenance = serde_json::json!({
        "schema_version": 1,
        "generated_by": "bijux-dev-atlas release sign",
        "release_id": release_id,
        "git_sha": git_sha,
        "governance_version": governance_version,
        "toolchain_inventory": "configs/sources/repository/rust-tooling/toolchain.json",
        "signing_policy_path": repo_rel(&root, &policy_path),
        "evidence_manifest_path": repo_rel(&root, &manifest_path),
        "checksum_list_path": repo_rel(&root, &checksums_path)
    });
    write_json(&provenance_path, &provenance)?;
    manifest["provenance"] = serde_json::json!({
        "path": repo_rel(&root, &provenance_path),
        "sha256": sha256_file(&provenance_path)?
    });

    let checksums_rel = repo_rel(&root, &checksums_path);
    let provenance_rel = repo_rel(&root, &provenance_path);
    let sign_report_rel = repo_rel(&root, &sign_report_path);
    let verify_report_path = root.join("ops/release/signing/release-verify.json");
    let provisional_sign_report = serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "policy_path": repo_rel(&root, &policy_path),
        "checksums_path": checksums_rel,
        "provenance_path": provenance_rel,
        "signed_items": [],
        "verification_command": policy
            .get("verification")
            .and_then(|value| value.get("command"))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default(),
        "mechanism": policy
            .get("mechanism")
            .and_then(|value| value.get("type"))
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default(),
        "contracts": {
            "REL-SIGN-001": false,
            "REL-SIGN-002": false,
            "REL-SIGN-003": false,
            "REL-SIGN-006": false
        },
        "errors": []
    });
    write_json(&sign_report_path, &provisional_sign_report)?;

    let mut signature_artifacts = vec![
        checksums_rel.clone(),
        provenance_rel.clone(),
        sign_report_rel.clone(),
    ];
    if verify_report_path.exists() {
        signature_artifacts.push(repo_rel(&root, &verify_report_path));
    }
    signature_artifacts.sort();
    signature_artifacts.dedup();
    manifest["signature_artifacts"] = serde_json::json!(signature_artifacts);
    write_json(&manifest_path, &manifest)?;
    let tarball_path = root.join("ops/release/evidence/bundle.tar");
    let tar_members = collect_tarball_members(&root, &manifest)?;
    build_normalized_tarball(&root, &tarball_path, &tar_members)?;
    manifest["evidence_tarball"] = serde_json::json!({
        "path": repo_rel(&root, &tarball_path),
        "sha256": sha256_file(&tarball_path)?
    });
    write_json(&manifest_path, &manifest)?;

    let mut checksum_items = Vec::new();
    for item in &signed_items {
        let path = item
            .get("path")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        let kind = item
            .get("kind")
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or_default();
        if path.is_empty() || kind.is_empty() {
            errors.push("signing policy contains an incomplete signed item".to_string());
            continue;
        }
        let abs = root.join(path);
        if !abs.exists() {
            errors.push(format!("signed item does not exist: {path}"));
            continue;
        }
        checksum_items.push(serde_json::json!({
            "path": path,
            "kind": kind,
            "sha256": sha256_file(&abs)?
        }));
    }

    let checksum_list = serde_json::json!({
        "schema_version": 1,
        "generated_by": "bijux-dev-atlas release sign",
        "items": checksum_items
    });
    write_json(&checksums_path, &checksum_list)?;

    let verification_command = policy
        .get("verification")
        .and_then(|value| value.get("command"))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mechanism = policy
        .get("mechanism")
        .and_then(|value| value.get("type"))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let sign_report = serde_json::json!({
        "schema_version": 1,
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "policy_path": repo_rel(&root, &policy_path),
        "checksums_path": checksums_rel,
        "provenance_path": provenance_rel,
        "signed_items": checksum_list["items"].clone(),
        "verification_command": verification_command,
        "mechanism": mechanism,
        "contracts": {
            "REL-SIGN-001": errors.is_empty(),
            "REL-SIGN-002": errors.is_empty(),
            "REL-SIGN-003": errors.is_empty(),
            "REL-SIGN-006": errors.is_empty()
        },
        "errors": errors
    });
    write_json(&sign_report_path, &sign_report)?;

    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": sign_report["status"].clone(),
            "text": if sign_report["status"] == "ok" { "release signing artifacts generated" } else { "release signing failed" },
            "rows": [{
                "report_path": repo_rel(&root, &sign_report_path),
                "checksums_path": sign_report["checksums_path"].clone(),
                "provenance_path": sign_report["provenance_path"].clone(),
                "contracts": sign_report["contracts"].clone(),
                "errors": sign_report["errors"].clone()
            }],
            "summary": {"total": 1, "errors": if sign_report["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((rendered, if sign_report["status"] == "ok" { 0 } else { 1 }))
}

fn run_release_verify(args: ReleaseVerifyArgs) -> Result<(String, i32), String> {
    let repo_root_arg = args.repo_root.clone();
    let root = resolve_repo_root(repo_root_arg.clone())?;
    ensure_json(&root.join("configs/schemas/contracts/release/checksum-list.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/release/release-sign.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/release/release-verify.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/release/provenance.schema.json"))?;

    let tarball = resolve_release_evidence_tarball(&root, &args.evidence);
    let checksums_path = root.join("ops/release/signing/checksums.json");
    let sign_report_path = root.join("ops/release/signing/release-sign.json");
    let verify_report_path = root.join("ops/release/signing/release-verify.json");
    let provenance_path = root.join("ops/release/provenance.json");
    let policy_path = root.join("ops/release/signing/policy.yaml");

    let checksums = read_json(&checksums_path)?;
    let sign_report = read_json(&sign_report_path)?;
    let provenance = read_json(&provenance_path)?;
    let policy = read_yaml(&policy_path)?;
    let manifest = read_json(&root.join("ops/release/evidence/manifest.json"))?;
    let manifest_schema = read_json(&root.join("ops/release/evidence/manifest.schema.json"))?;

    let checksum_items = checksums
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let sign_items = sign_report
        .get("signed_items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let policy_items = policy
        .get("signed_items")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let rel_sign_001 = !checksum_items.is_empty() && checksum_items.len() == policy_items.len();
    let mut checksum_errors = Vec::new();
    for item in &checksum_items {
        let path = item
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let expected = item
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let abs = root.join(path);
        if path.is_empty() || expected.is_empty() || !abs.exists() {
            checksum_errors.push(format!("missing checksum target: {path}"));
            continue;
        }
        let actual = sha256_file(&abs)?;
        if actual != expected {
            checksum_errors.push(format!("checksum mismatch: {path}"));
        }
    }
    let rel_sign_002 = checksum_errors.is_empty();
    let rel_sign_003 = sign_report
        .get("status")
        .and_then(serde_json::Value::as_str)
        == Some("ok")
        && !sign_items.is_empty()
        && sign_items.len() == policy_items.len();
    let signature_artifacts = manifest
        .get("signature_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let expected_signature_artifacts = [
        "ops/release/signing/checksums.json",
        "ops/release/signing/release-sign.json",
        "ops/release/provenance.json",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let manifest_signature_artifacts = signature_artifacts
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<BTreeSet<_>>();
    let rel_sign_005 = expected_signature_artifacts
        .iter()
        .all(|item| manifest_signature_artifacts.contains(item));
    let rel_sign_006 = policy_items.iter().all(|item| {
        item.get("path")
            .and_then(serde_yaml::Value::as_str)
            .is_some_and(|path| root.join(path).exists())
    });
    let rel_prov_001 = provenance
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        == Some(1)
        && provenance
            .get("git_sha")
            .and_then(serde_json::Value::as_str)
            .is_some();
    let rel_man_001 = manifest
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        == manifest_schema
            .get("properties")
            .and_then(|value| value.get("schema_version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_i64);
    let temp_tarball = root.join("artifacts/release/repro-check.tar");
    if let Some(parent) = temp_tarball.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let rebuild_members = collect_tarball_members(&root, &manifest)?;
    build_normalized_tarball(&root, &temp_tarball, &rebuild_members)?;
    let original_members = tarball_member_checksums(&tarball)?;
    let rebuilt_members = tarball_member_checksums(&temp_tarball)?;
    let differing_paths = original_members
        .keys()
        .chain(rebuilt_members.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| original_members.get(path) != rebuilt_members.get(path))
        .collect::<Vec<_>>();
    let allowed_repro_differences = [
        "ops/release/evidence/manifest.json",
        "ops/release/signing/checksums.json",
        "ops/release/signing/release-sign.json",
        "ops/release/signing/release-verify.json",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let rel_tar_001 = differing_paths
        .iter()
        .all(|path| allowed_repro_differences.contains(path.as_str()));
    let _ = fs::remove_file(&temp_tarball);

    let exe = std::env::current_exe().map_err(|err| format!("release verify failed: {err}"))?;
    let mut evidence_args = vec![
        "ops".to_string(),
        "evidence".to_string(),
        "verify".to_string(),
        tarball.display().to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    if let Some(repo_root) = &repo_root_arg {
        evidence_args.push("--repo-root".to_string());
        evidence_args.push(repo_root.display().to_string());
    }
    let evidence_out = ProcessCommand::new(&exe)
        .args(&evidence_args)
        .output()
        .map_err(|err| format!("release verify failed: {err}"))?;
    let evidence_ok = evidence_out.status.success();
    let evidence_payload: serde_json::Value =
        serde_json::from_slice(&evidence_out.stdout).unwrap_or_else(|_| {
            serde_json::json!({"status":"failed","stderr": String::from_utf8_lossy(&evidence_out.stderr)})
        });
    let readiness_report_path =
        root.join("artifacts/ops/ops_run/observe/operational-readiness-report.json");
    let rel_ops_001 = std::fs::read_to_string(&readiness_report_path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .is_some_and(|report| {
            report.get("status").and_then(serde_json::Value::as_str) == Some("ok")
                && report
                    .get("completeness")
                    .and_then(serde_json::Value::as_f64)
                    .is_some_and(|value| value >= 1.0)
        });
    let rel_sign_004 = rel_sign_001
        && rel_sign_002
        && rel_sign_003
        && rel_sign_005
        && rel_sign_006
        && rel_prov_001
        && rel_man_001
        && rel_tar_001
        && evidence_ok
        && rel_ops_001;
    let mut errors = checksum_errors;
    if !evidence_ok {
        errors.push("ops evidence verify failed".to_string());
    }
    if !rel_sign_001 {
        errors.push("checksum list is incomplete".to_string());
    }
    if !rel_sign_003 {
        errors.push("sign report is incomplete".to_string());
    }
    if !rel_sign_005 {
        errors.push("manifest does not list all signature artifacts".to_string());
    }
    if !rel_sign_006 {
        errors.push("signing policy references a missing artifact".to_string());
    }
    if !rel_prov_001 {
        errors.push("provenance file is missing required fields".to_string());
    }
    if !rel_man_001 {
        errors.push("manifest schema version does not match the governed schema".to_string());
    }
    if !rel_tar_001 {
        errors.push(format!(
            "reproducible tarball check failed for: {}",
            differing_paths.join(", ")
        ));
    }
    if !rel_ops_001 {
        errors.push(format!(
            "operational readiness report is missing or below threshold: {}",
            readiness_report_path.display()
        ));
    }

    let verify_report = serde_json::json!({
        "schema_version": 1,
        "status": if rel_sign_004 { "ok" } else { "failed" },
        "evidence_tarball": repo_rel(&root, &tarball),
        "checksums_path": repo_rel(&root, &checksums_path),
        "contracts": {
            "REL-SIGN-001": rel_sign_001,
            "REL-SIGN-002": rel_sign_002,
            "REL-SIGN-003": rel_sign_003,
            "REL-SIGN-004": rel_sign_004,
            "REL-SIGN-005": rel_sign_005,
            "REL-SIGN-006": rel_sign_006,
            "REL-TAR-001": rel_tar_001,
            "REL-MAN-001": rel_man_001,
            "REL-PROV-001": rel_prov_001,
            "REL-OPS-001": rel_ops_001
        },
        "errors": errors
    });
    write_json(&verify_report_path, &verify_report)?;

    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": verify_report["status"].clone(),
            "text": if verify_report["status"] == "ok" { "release verification succeeded" } else { "release verification failed" },
            "rows": [{
                "report_path": repo_rel(&root, &verify_report_path),
                "contracts": verify_report["contracts"].clone(),
                "errors": verify_report["errors"].clone(),
                "evidence_verify": evidence_payload
            }],
            "summary": {"total": 1, "errors": if verify_report["status"] == "ok" { 0 } else { 1 }, "warnings": 0}
        }),
    )?;
    Ok((
        rendered,
        if verify_report["status"] == "ok" {
            0
        } else {
            1
        },
    ))
}

fn resolve_release_evidence_tarball(root: &Path, evidence: &Path) -> PathBuf {
    let candidate = if evidence.is_absolute() {
        evidence.to_path_buf()
    } else {
        root.join(evidence)
    };
    if candidate.is_dir() {
        candidate.join("bundle.tar")
    } else {
        candidate
    }
}

fn classify_release_diff(path: &str) -> &'static str {
    if path.starts_with("ops/release/signing/") || path == "ops/release/provenance.json" {
        "signing"
    } else if path.starts_with("ops/release/evidence/sboms/") {
        "sbom"
    } else if path.starts_with("ops/release/evidence/packages/") {
        "artifact"
    } else if path.starts_with("ops/release/evidence/") {
        "evidence"
    } else {
        "supporting"
    }
}

fn run_release_diff(args: ReleaseDiffArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let tarball_a = if args.evidence_a.is_absolute() {
        args.evidence_a
    } else {
        root.join(args.evidence_a)
    };
    let tarball_b = if args.evidence_b.is_absolute() {
        args.evidence_b
    } else {
        root.join(args.evidence_b)
    };
    let members_a = tarball_member_checksums(&tarball_a)?;
    let members_b = tarball_member_checksums(&tarball_b)?;
    let names = members_a
        .keys()
        .chain(members_b.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for name in names {
        match (members_a.get(&name), members_b.get(&name)) {
            (None, Some(_)) => added.push(serde_json::json!({
                "path": name,
                "class": classify_release_diff(&name)
            })),
            (Some(_), None) => removed.push(serde_json::json!({
                "path": name,
                "class": classify_release_diff(&name)
            })),
            (Some(left), Some(right)) if left != right => changed.push(serde_json::json!({
                "path": name,
                "class": classify_release_diff(&name),
                "sha256_a": left,
                "sha256_b": right
            })),
            _ => {}
        }
    }
    let differences = !added.is_empty() || !removed.is_empty() || !changed.is_empty();
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": "ok",
        "text": if differences { "release bundles differ" } else { "release bundles match" },
        "rows": [{
            "evidence_a": tarball_a.display().to_string(),
            "evidence_b": tarball_b.display().to_string(),
            "added": added,
            "removed": removed,
            "changed": changed
        }],
        "summary": {
            "total": 1,
            "errors": 0,
            "warnings": 0,
            "differences": if differences { 1 } else { 0 }
        }
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, 0))
}

fn run_release_packet(args: ReleasePacketArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/schemas/contracts/release/packet-list.schema.json"))?;

    let evidence_dir = if args.evidence.is_absolute() {
        args.evidence
    } else {
        root.join(args.evidence)
    };
    let manifest_path = evidence_dir.join("manifest.json");
    let manifest = read_json(&manifest_path)?;
    let packet_dir = root.join("ops/release/packet");
    let packet_path = packet_dir.join("packet.json");

    let required = [
        "ops/release/evidence/manifest.json",
        "ops/release/evidence/identity.json",
        "ops/release/evidence/bundle.tar",
        "ops/release/signing/checksums.json",
        "ops/release/signing/release-sign.json",
        "ops/release/signing/release-verify.json",
        "ops/release/provenance.json",
    ];

    let mut selected = BTreeSet::new();
    for item in required {
        if root.join(item).exists() {
            selected.insert(item.to_string());
        }
    }
    if let Some(path) = manifest
        .get("chart_package")
        .and_then(|value| value.get("path"))
        .and_then(serde_json::Value::as_str)
    {
        if root.join(path).exists() {
            selected.insert(path.to_string());
        }
    }
    for path in manifest
        .get("sboms")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| value.get("path"))
        .filter_map(serde_json::Value::as_str)
    {
        if root.join(path).exists() {
            selected.insert(path.to_string());
        }
    }
    for path in manifest
        .get("dataset_assets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        if root.join(path).exists() {
            selected.insert(path.to_string());
        }
    }

    let packet_items = selected
        .iter()
        .map(|path| {
            serde_json::json!({
                "path": path,
                "sha256": sha256_file(&root.join(path)).unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();

    let rel_pack_001 = required.iter().all(|path| selected.contains(*path))
        && packet_items.iter().any(|item| {
            item["path"]
                .as_str()
                .is_some_and(|path| path.starts_with("ops/release/evidence/sboms/"))
        });

    let packet = serde_json::json!({
        "schema_version": 1,
        "generated_by": "bijux-dev-atlas release packet",
        "evidence_root": repo_rel(&root, &evidence_dir),
        "required_minimum": required,
        "items": packet_items,
        "contracts": {
            "REL-PACK-001": rel_pack_001
        }
    });
    write_json(&packet_path, &packet)?;

    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": if rel_pack_001 { "ok" } else { "failed" },
            "text": if rel_pack_001 { "institutional packet inventory generated" } else { "institutional packet is incomplete" },
            "rows": [{
                "packet_path": repo_rel(&root, &packet_path),
                "items": packet["items"].clone(),
                "contracts": packet["contracts"].clone()
            }],
            "summary": {
                "total": 1,
                "errors": if rel_pack_001 { 0 } else { 1 },
                "warnings": 0
            }
        }),
    )?;
    Ok((rendered, if rel_pack_001 { 0 } else { 1 }))
}

fn default_release_version(root: &Path) -> String {
    if let Some(version) = env_var_text("GITHUB_REF")
        .and_then(|value| value.strip_prefix("refs/tags/").map(str::to_string))
        .and_then(|tag| crate::version_support::version_from_tag(&tag))
    {
        return version;
    }
    let chart_yaml = root.join("ops/k8s/charts/bijux-atlas/Chart.yaml");
    if let Ok(value) = read_yaml(&chart_yaml) {
        if let Some(version) = value.get("version").and_then(serde_yaml::Value::as_str) {
            if !version.trim().is_empty() {
                return version.to_string();
            }
        }
    }
    if let Some(version) = crate::version_support::latest_version_tag(root)
        .and_then(|tag| crate::version_support::version_from_tag(&tag))
    {
        return version;
    }
    "0.0.0".to_string()
}

fn release_line_for_version(version: &str) -> String {
    crate::version_support::release_line_from_version(version)
        .unwrap_or_else(|| "unknown".to_string())
}

fn release_line_from_spec(spec: &toml::Value, root: &Path) -> String {
    spec.get("release_line")
        .and_then(toml::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| release_line_for_version(&default_release_version(root)))
}

fn release_root(root: &Path, version: &str) -> std::path::PathBuf {
    root.join("artifacts/release").join(version)
}

fn release_manifest_path(root: &Path, version: &str) -> std::path::PathBuf {
    release_root(root, version).join("manifest.json")
}

fn release_bundle_hash(members: &[serde_json::Value]) -> String {
    let mut hasher = Sha256::new();
    for row in members {
        let path = row
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let sha256 = row
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let size = row
            .get("size_bytes")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        hasher.update(path.as_bytes());
        hasher.update(b"\n");
        hasher.update(sha256.as_bytes());
        hasher.update(b"\n");
        hasher.update(size.to_string().as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

fn collect_manifest_source(root: &Path) -> Result<serde_json::Value, String> {
    read_json(&root.join("ops/release/evidence/manifest.json"))
}

fn collect_toolchain_versions(root: &Path) -> serde_json::Value {
    let path = root.join("configs/sources/repository/rust-tooling/toolchain.json");
    let value = read_json(&path).unwrap_or(serde_json::Value::Null);
    value
        .get("versions")
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}

fn read_reproducibility_policy(root: &Path) -> Result<serde_json::Value, String> {
    read_json(&root.join("configs/sources/release/reproducibility-policy.json"))
}

fn create_release_manifest(root: &Path, version: &str) -> Result<serde_json::Value, String> {
    let source = collect_manifest_source(root)?;
    let git_sha = ProcessCommand::new("git")
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let build_time = env_var_text("SOURCE_DATE_EPOCH").unwrap_or_else(|| "0".to_string());
    let docs_hash = source
        .get("docs_site_summary")
        .and_then(|v| v.get("sha256"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let ops_profiles = source
        .get("image_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            row.get("profile")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let sbom_refs = source
        .get("sboms")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "path": row.get("path").cloned().unwrap_or(serde_json::Value::Null),
                "sha256": row.get("sha256").cloned().unwrap_or(serde_json::Value::Null)
            })
        })
        .collect::<Vec<_>>();
    let image_digests = source
        .get("image_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            row.get("digest")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .filter(|digest| !digest.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let chart_path = source
        .get("chart_package")
        .and_then(|v| v.get("path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let chart_digest = source
        .get("chart_package")
        .and_then(|v| v.get("sha256"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut payload = serde_json::json!({
        "schema_version": 1,
        "version": version,
        "git_sha": git_sha,
        "build_time": build_time,
        "control_plane_version": bijux_dev_atlas::version::runtime_version(),
        "ops_profiles_validated": ops_profiles,
        "docs_build_hash": docs_hash,
        "sbom_digests": sbom_refs,
        "container_image_digests": image_digests,
        "chart": {
            "version": version,
            "path": chart_path,
            "sha256": chart_digest
        },
        "build_metadata": {
            "os": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "cargo_profile": env_var_text("PROFILE").unwrap_or_else(|| "release".to_string()),
            "toolchain_versions": collect_toolchain_versions(root)
        },
        "artifact_list": [],
        "security_advisories": []
    });
    payload["artifact_count"] = serde_json::json!(0);
    payload["artifact_total_size_bytes"] = serde_json::json!(0);
    Ok(payload)
}

fn required_release_tree_entries() -> BTreeSet<&'static str> {
    [
        "manifest.json",
        "images",
        "charts",
        "docs",
        "sbom",
        "provenance",
    ]
    .into_iter()
    .collect()
}

fn run_release_manifest_generate(
    args: ReleaseManifestGenerateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let release_dir = release_root(&root, &version);
    fs::create_dir_all(&release_dir)
        .map_err(|err| format!("failed to create {}: {err}", release_dir.display()))?;
    let manifest = create_release_manifest(&root, &version)?;
    let path = release_manifest_path(&root, &version);
    write_json(&path, &manifest)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "release_manifest_generate",
            "status": "ok",
            "version": version,
            "path": repo_rel(&root, &path)
        }),
    )?;
    Ok((rendered, 0))
}

fn validate_release_manifest(root: &Path, version: &str) -> Result<serde_json::Value, String> {
    ensure_json(&root.join("configs/schemas/contracts/release/release-manifest.schema.json"))?;
    let manifest_path = release_manifest_path(root, version);
    let manifest = read_json(&manifest_path)?;
    let mut errors = Vec::<String>::new();
    for key in [
        "version",
        "git_sha",
        "build_time",
        "artifact_list",
        "control_plane_version",
        "ops_profiles_validated",
        "docs_build_hash",
        "sbom_digests",
        "container_image_digests",
        "chart",
        "build_metadata",
    ] {
        if manifest.get(key).is_none() {
            errors.push(format!("missing required key `{key}`"));
        }
    }
    let artifact_list = manifest
        .get("artifact_list")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    for row in &artifact_list {
        let path = row
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let expected = row
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if path.is_empty() || expected.is_empty() {
            errors.push("artifact_list row requires path and sha256".to_string());
            continue;
        }
        let abs = release_root(root, version).join(path);
        if !abs.exists() {
            errors.push(format!("artifact does not exist: {path}"));
            continue;
        }
        let actual = sha256_file(&abs)?;
        if actual != expected {
            errors.push(format!("artifact digest mismatch: {path}"));
        }
    }
    let source = collect_manifest_source(root)?;
    let known_image_digests = source
        .get("image_artifacts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            row.get("digest")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    for digest in manifest
        .get("container_image_digests")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
    {
        if !known_image_digests.contains(digest.as_str()) {
            errors.push(format!(
                "container digest not present in source evidence: {digest}"
            ));
        }
    }
    let chart_path = manifest
        .get("chart")
        .and_then(|v| v.get("path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let chart_sha = manifest
        .get("chart")
        .and_then(|v| v.get("sha256"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if !chart_path.is_empty() {
        let abs = root.join(chart_path);
        if !abs.exists() {
            errors.push(format!("chart package missing: {chart_path}"));
        } else if !chart_sha.is_empty() && sha256_file(&abs)? != chart_sha {
            errors.push("chart digest does not match packaged chart".to_string());
        }
    }
    let has_docs = artifact_list.iter().any(|row| {
        row.get("path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|p| p.starts_with("docs/"))
    });
    if !has_docs {
        errors.push("release bundle must include docs artifact".to_string());
    }
    let has_ops_evidence = artifact_list.iter().any(|row| {
        row.get("path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|p| p.contains("ops-profile"))
    });
    if !has_ops_evidence {
        errors.push("release bundle must include ops profile evidence".to_string());
    }
    let has_sbom = artifact_list.iter().any(|row| {
        row.get("path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|p| p.starts_with("sbom/"))
    });
    if !has_sbom {
        errors.push("release bundle must include sbom artifact".to_string());
    }
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "release_manifest_validate",
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "version": version,
        "path": repo_rel(root, &manifest_path),
        "errors": errors
    }))
}

fn run_release_manifest_validate(
    args: ReleaseManifestValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let payload = validate_release_manifest(&root, &version)?;
    let code = if payload["status"] == "ok" { 0 } else { 1 };
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, code))
}

fn run_release_bundle_build(args: ReleaseBundleBuildArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let out_root = release_root(&root, &version);
    let images_dir = out_root.join("images");
    let charts_dir = out_root.join("charts");
    let docs_dir = out_root.join("docs");
    let sbom_dir = out_root.join("sbom");
    let provenance_dir = out_root.join("provenance");
    for dir in [
        &images_dir,
        &charts_dir,
        &docs_dir,
        &sbom_dir,
        &provenance_dir,
    ] {
        fs::create_dir_all(dir)
            .map_err(|err| format!("failed to create {}: {err}", dir.display()))?;
    }
    let source = collect_manifest_source(&root)?;
    let mut copied = Vec::<serde_json::Value>::new();
    let image_digest_path = images_dir.join("container-image-digests.json");
    let image_digest_payload = serde_json::json!({
        "schema_version": 1,
        "digests": source.get("image_artifacts").cloned().unwrap_or(serde_json::Value::Null)
    });
    write_json(&image_digest_path, &image_digest_payload)?;
    copied.push(serde_json::json!({"path":"images/container-image-digests.json","sha256":sha256_file(&image_digest_path)?,"size_bytes":fs::metadata(&image_digest_path).map_err(|e| e.to_string())?.len()}));

    let docs_hash = source
        .get("docs_site_summary")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let docs_hash_path = docs_dir.join("docs-build.json");
    write_json(&docs_hash_path, &docs_hash)?;
    copied.push(serde_json::json!({"path":"docs/docs-build.json","sha256":sha256_file(&docs_hash_path)?,"size_bytes":fs::metadata(&docs_hash_path).map_err(|e| e.to_string())?.len()}));

    let ops_profiles = source
        .get("image_artifacts")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let ops_profile_path = provenance_dir.join("ops-profile-evidence.json");
    write_json(&ops_profile_path, &ops_profiles)?;
    copied.push(serde_json::json!({"path":"provenance/ops-profile-evidence.json","sha256":sha256_file(&ops_profile_path)?,"size_bytes":fs::metadata(&ops_profile_path).map_err(|e| e.to_string())?.len()}));

    let provenance_src = root.join("ops/release/provenance.json");
    if provenance_src.exists() {
        let dst = provenance_dir.join("provenance.json");
        fs::copy(&provenance_src, &dst)
            .map_err(|err| format!("failed to copy {}: {err}", provenance_src.display()))?;
        copied.push(serde_json::json!({"path":"provenance/provenance.json","sha256":sha256_file(&dst)?,"size_bytes":fs::metadata(&dst).map_err(|e| e.to_string())?.len()}));
    }

    if let Some(chart_path) = source
        .get("chart_package")
        .and_then(|v| v.get("path"))
        .and_then(serde_json::Value::as_str)
    {
        let src = root.join(chart_path);
        if src.exists() {
            let file = src
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("chart.tgz");
            let dst = charts_dir.join(file);
            fs::copy(&src, &dst)
                .map_err(|err| format!("failed to copy {}: {err}", src.display()))?;
            copied.push(serde_json::json!({"path":format!("charts/{file}"),"sha256":sha256_file(&dst)?,"size_bytes":fs::metadata(&dst).map_err(|e| e.to_string())?.len()}));
        }
    }
    for row in source
        .get("sboms")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(src_rel) = row.get("path").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let src = root.join(src_rel);
        if !src.exists() {
            continue;
        }
        let file = src
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("sbom.json");
        let dst = sbom_dir.join(file);
        fs::copy(&src, &dst).map_err(|err| format!("failed to copy {}: {err}", src.display()))?;
        copied.push(serde_json::json!({"path":format!("sbom/{file}"),"sha256":sha256_file(&dst)?,"size_bytes":fs::metadata(&dst).map_err(|e| e.to_string())?.len()}));
    }
    copied.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let mut manifest = create_release_manifest(&root, &version)?;
    manifest["artifact_list"] = serde_json::json!(copied);
    manifest["artifact_count"] = serde_json::json!(manifest["artifact_list"]
        .as_array()
        .map(|r| r.len())
        .unwrap_or(0));
    manifest["artifact_total_size_bytes"] = serde_json::json!(manifest["artifact_list"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r.get("size_bytes").and_then(serde_json::Value::as_u64))
        .sum::<u64>());
    let bundle_hash = release_bundle_hash(
        manifest["artifact_list"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .as_slice(),
    );
    manifest["bundle_hash"] = serde_json::json!(bundle_hash.clone());
    let manifest_path = release_manifest_path(&root, &version);
    write_json(&manifest_path, &manifest)?;
    fs::write(out_root.join("bundle.sha256"), format!("{bundle_hash}\n"))
        .map_err(|err| format!("failed to write bundle hash: {err}"))?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "release_bundle_build",
            "status": "ok",
            "version": version,
            "root": repo_rel(&root, &out_root),
            "bundle_hash": bundle_hash
        }),
    )?;
    Ok((rendered, 0))
}

fn run_release_bundle_hash(args: ReleaseBundleHashArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let manifest = read_json(&release_manifest_path(&root, &version))?;
    let items = manifest
        .get("artifact_list")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let bundle_hash = release_bundle_hash(items.as_slice());
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "release_bundle_hash",
            "status": "ok",
            "version": version,
            "bundle_hash": bundle_hash
        }),
    )?;
    Ok((rendered, 0))
}

fn run_release_bundle_verify(args: ReleaseBundleVerifyArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let bundle_root = release_root(&root, &version);
    let mut errors = Vec::<String>::new();
    let required = required_release_tree_entries();
    for item in &required {
        if !bundle_root.join(item).exists() {
            errors.push(format!("missing required release tree entry `{item}`"));
        }
    }
    if bundle_root.exists() {
        for entry in fs::read_dir(&bundle_root)
            .map_err(|err| format!("failed to read {}: {err}", bundle_root.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read release tree entry: {err}"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !required.contains(name.as_str())
                && name != "bundle.sha256"
                && name != "launch-checklist.md"
                && name != "readiness-report.json"
            {
                errors.push(format!("unexpected file in release tree: {name}"));
            }
        }
    }
    let manifest_payload = validate_release_manifest(&root, &version)?;
    if manifest_payload["status"] != "ok" {
        errors.extend(
            manifest_payload["errors"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| value.as_str().map(str::to_string)),
        );
    }
    let hash_payload = run_release_bundle_hash(ReleaseBundleHashArgs {
        repo_root: Some(root.clone()),
        version: Some(version.clone()),
        format: FormatArg::Json,
        out: None,
    })?;
    let computed_hash: serde_json::Value =
        serde_json::from_str(&hash_payload.0).unwrap_or_default();
    let computed = computed_hash
        .get("bundle_hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let manifest = read_json(&release_manifest_path(&root, &version))?;
    let declared = manifest
        .get("bundle_hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    if !declared.is_empty() && computed != declared {
        errors.push("same commit should produce identical bundle hash".to_string());
    }
    if !release_manifest_path(&root, &version).exists() {
        errors.push("release bundle contains no manifest".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "release_bundle_verify",
            "status": status,
            "version": version,
            "bundle_hash": computed,
            "errors": errors
        }),
    )?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_rebuild_verify(args: ReleaseRebuildVerifyArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let manifest = read_json(&release_manifest_path(&root, &version))?;
    let declared_hash = manifest
        .get("bundle_hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let hash_payload = run_release_bundle_hash(ReleaseBundleHashArgs {
        repo_root: Some(root.clone()),
        version: Some(version.clone()),
        format: FormatArg::Json,
        out: None,
    })?;
    let computed: serde_json::Value =
        serde_json::from_str(&hash_payload.0).unwrap_or_else(|_| serde_json::json!({}));
    let computed_hash = computed
        .get("bundle_hash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut errors = Vec::<String>::new();
    if declared_hash.is_empty() {
        errors.push("manifest is missing bundle_hash".to_string());
    } else if declared_hash != computed_hash {
        errors.push("rebuild hash must equal original bundle hash".to_string());
    }
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_rebuild_verify",
        "status": if errors.is_empty() { "ok" } else { "failed" },
        "version": version,
        "declared_bundle_hash": declared_hash,
        "computed_bundle_hash": computed_hash,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if payload["status"] == "ok" { 0 } else { 1 }))
}

fn run_release_reproducibility_report(
    args: ReleaseReproducibilityReportArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let policy = read_reproducibility_policy(&root)?;
    let required_env = policy
        .get("required_env")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut env_results = Vec::<serde_json::Value>::new();
    let mut errors = Vec::<String>::new();
    for (key, expected) in required_env {
        let expected_value = expected.as_str().unwrap_or_default().to_string();
        let actual_value = env_var_text(&key).unwrap_or_default();
        let ok = actual_value == expected_value;
        if !ok {
            errors.push(format!(
                "build environment mismatch for `{key}`: expected `{expected_value}`, got `{actual_value}`"
            ));
        }
        env_results.push(serde_json::json!({
            "key": key,
            "expected": expected_value,
            "actual": actual_value,
            "status": if ok { "ok" } else { "failed" }
        }));
    }

    let manifest = read_json(&release_manifest_path(&root, &version))?;
    let has_build_metadata = manifest
        .get("build_metadata")
        .and_then(serde_json::Value::as_object)
        .is_some();
    if !has_build_metadata {
        errors.push("release manifest is missing build_metadata".to_string());
    }

    let rebuild_payload = run_release_rebuild_verify(ReleaseRebuildVerifyArgs {
        repo_root: Some(root.clone()),
        version: Some(version.clone()),
        format: FormatArg::Json,
        out: None,
    })?;
    let rebuild_report: serde_json::Value =
        serde_json::from_str(&rebuild_payload.0).unwrap_or_else(|_| serde_json::json!({}));
    if policy
        .get("require_rebuild_hash_match")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
        && rebuild_report
            .get("status")
            .and_then(serde_json::Value::as_str)
            != Some("ok")
    {
        errors.push("rebuild hash must equal original bundle hash".to_string());
    }

    let status = if errors.is_empty() { "ok" } else { "failed" };
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "release_reproducibility_report",
        "status": status,
        "version": version,
        "environment": env_results,
        "rebuild": rebuild_report,
        "manifest_build_metadata_present": has_build_metadata,
        "errors": errors
    });
    let report_path = root.join("artifacts/release/reproducibility-report.json");
    write_json(&report_path, &report)?;
    let mut response = report.clone();
    response["path"] = serde_json::json!(repo_rel(&root, &report_path));
    let rendered = emit_payload(args.format, args.out, &response)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

#[derive(Debug, Clone)]
struct SemverLike {
    major: u64,
    minor: u64,
    patch: u64,
    prerelease: Option<String>,
}

fn parse_semver_like(value: &str) -> Option<SemverLike> {
    let mut base_and_pre = value.splitn(2, '-');
    let base = base_and_pre.next()?.trim();
    let prerelease = base_and_pre.next().map(str::to_string);
    let mut parts = base.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(SemverLike {
        major,
        minor,
        patch,
        prerelease,
    })
}

fn version_from_tag(value: &str) -> String {
    value.strip_prefix('v').unwrap_or(value).to_string()
}

fn extract_changelog_versions(changelog: &str) -> Vec<String> {
    changelog
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("## "))
        .map(|line| line.trim())
        .filter_map(|line| line.strip_prefix('v'))
        .map(str::to_string)
        .collect()
}

fn release_manifest_version(root: &Path, expected: &str) -> Option<String> {
    let path = release_manifest_path(root, expected);
    if !path.exists() {
        return None;
    }
    read_json(&path).ok().and_then(|value| {
        value
            .get("version")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    })
}

fn generate_release_docs_artifacts(
    root: &Path,
    version: &str,
    versions: &[String],
) -> Result<(), String> {
    let generated_dir = root.join("ops/release/generated");
    fs::create_dir_all(&generated_dir)
        .map_err(|err| format!("failed to create {}: {err}", generated_dir.display()))?;
    let metadata_path = generated_dir.join("release-metadata.json");
    let metadata = serde_json::json!({
        "schema_version": 1,
        "kind": "release_metadata",
        "version": version,
        "artifact_root": format!("artifacts/release/{version}"),
        "manifest_path": format!("artifacts/release/{version}/manifest.json")
    });
    write_json(&metadata_path, &metadata)?;

    let mut index = String::from("# Release Index\n\n");
    for v in versions {
        index.push_str(&format!("- v{v}: `artifacts/release/{v}/manifest.json`\n"));
    }
    fs::write(generated_dir.join("release-index.md"), index)
        .map_err(|err| format!("failed to write release index page: {err}"))?;

    let mut feed = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"><channel>\n",
    );
    feed.push_str("<title>bijux-atlas releases</title>\n");
    for v in versions {
        feed.push_str(&format!(
            "<item><title>v{v}</title><link>artifacts/release/{v}/manifest.json</link></item>\n"
        ));
    }
    feed.push_str("</channel></rss>\n");
    fs::write(generated_dir.join("release-feed.xml"), feed)
        .map_err(|err| format!("failed to write release feed: {err}"))?;
    Ok(())
}

fn run_release_version_check(args: ReleaseVersionCheckArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let policy_path = root.join("configs/sources/release/version-policy.json");
    let policy = read_json(&policy_path)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let tag = args.tag.or_else(|| {
        env_var_text("GITHUB_REF").and_then(|v| v.strip_prefix("refs/tags/").map(str::to_string))
    });
    let mut errors = Vec::<String>::new();
    let Some(parsed) = parse_semver_like(&version) else {
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "release_version_check",
            "status": "failed",
            "version": version,
            "errors": ["version is not valid semver"]
        });
        let rendered = emit_payload(args.format, args.out, &payload)?;
        return Ok((rendered, 1));
    };

    let allow_tags = policy
        .get("versioning")
        .and_then(|v| v.get("allow_prerelease_tags"))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();
    if let Some(pre) = &parsed.prerelease {
        let token = pre.split('.').next().unwrap_or_default().to_string();
        if !allow_tags.contains(token.as_str()) {
            errors.push(format!("prerelease tag `{token}` is not allowed"));
        }
        if token == "rc"
            && tag
                .as_ref()
                .is_some_and(|tag_value| !tag_value.contains("-rc"))
        {
            errors.push("release candidate versions must use release-candidate tags".to_string());
        }
    }

    if let Some(tag_value) = &tag {
        let require_v = policy
            .get("versioning")
            .and_then(|v| v.get("require_v_prefix_for_tags"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        if require_v && !tag_value.starts_with('v') {
            errors.push(format!("release tag `{tag_value}` must start with `v`"));
        }
        let tag_version = version_from_tag(tag_value);
        if tag_version != version {
            errors.push(format!(
                "release tag `{tag_value}` does not match version `{version}`"
            ));
        }
        if !version.contains("-rc") && tag_value.contains("-rc") {
            errors.push("release candidate tags require release candidate versions".to_string());
        }
    }

    if let Some(manifest_version) = release_manifest_version(&root, &version) {
        if manifest_version != version {
            errors.push(format!(
                "release manifest version `{manifest_version}` does not match `{version}`"
            ));
        }
    }

    let changelog_path = policy
        .get("changelog")
        .and_then(|v| v.get("path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("CHANGELOG.md");
    let changelog_text = fs::read_to_string(root.join(changelog_path))
        .map_err(|err| format!("failed to read {changelog_path}: {err}"))?;
    let versions = extract_changelog_versions(&changelog_text);
    if !versions.iter().any(|v| v == &version) {
        errors.push(format!("changelog must contain version `{version}`"));
    }
    if let Some(idx) = versions.iter().position(|v| v == &version) {
        if idx + 1 < versions.len() {
            if let (Some(prev), Some(cur)) = (
                parse_semver_like(&versions[idx + 1]),
                parse_semver_like(&version),
            ) {
                if (cur.major, cur.minor, cur.patch) <= (prev.major, prev.minor, prev.patch) {
                    errors.push("version bump cannot skip or move backwards".to_string());
                }
            }
        }
    }
    if release_root(&root, &version).exists() {
        let contract_payload = run_release_bundle_verify(ReleaseBundleVerifyArgs {
            repo_root: Some(root.clone()),
            version: Some(version.clone()),
            format: FormatArg::Json,
            out: None,
        })?;
        let contract: serde_json::Value =
            serde_json::from_str(&contract_payload.0).unwrap_or_else(|_| serde_json::json!({}));
        if contract.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            errors.push("version bump only allowed if release contract passes".to_string());
        }
    }

    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_version_check",
        "status": status,
        "version": version,
        "tag": tag,
        "policy_path": repo_rel(&root, &policy_path),
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn generate_release_readiness_report(
    root: &Path,
    version: &str,
) -> Result<serde_json::Value, String> {
    let manifest_check = validate_release_manifest(root, version)?;
    let bundle_check = run_release_bundle_verify(ReleaseBundleVerifyArgs {
        repo_root: Some(root.to_path_buf()),
        version: Some(version.to_string()),
        format: FormatArg::Json,
        out: None,
    })?;
    let bundle_payload: serde_json::Value =
        serde_json::from_str(&bundle_check.0).unwrap_or(serde_json::json!({"status":"failed"}));
    let release_dir = release_root(root, version);
    let required = [
        release_dir.join("manifest.json"),
        release_dir.join("bundle.sha256"),
        root.join("ops/release/evidence/bundle.tar"),
        root.join("ops/release/signing/checksums.json"),
        root.join("ops/release/provenance.json"),
    ];
    let mut missing_required = Vec::<String>::new();
    for path in required {
        if !path.exists() {
            missing_required.push(repo_rel(root, &path));
        }
    }
    let optional = [
        root.join("ops/release/ops-release-manifest.json"),
        root.join("ops/release/ops-chart-digest.json"),
        root.join("ops/report/generated/ops-release-readiness-summary.json"),
    ];
    let mut optional_rows = optional
        .iter()
        .map(|path| {
            serde_json::json!({
                "path": repo_rel(root, path),
                "status": if path.exists() { "present" } else { "missing" }
            })
        })
        .collect::<Vec<_>>();
    optional_rows.sort_by(|a, b| {
        a["path"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["path"].as_str().unwrap_or_default())
    });
    let status = if manifest_check["status"] == "ok"
        && bundle_payload["status"] == "ok"
        && missing_required.is_empty()
    {
        "ok"
    } else {
        "failed"
    };
    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "release_readiness_report",
        "status": status,
        "version": version,
        "manifest_check": manifest_check,
        "bundle_check": bundle_payload,
        "missing_required_artifacts": missing_required,
        "optional_artifacts": optional_rows
    }))
}

fn run_release_readiness_report(args: ReleaseBundleBuildArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let report = generate_release_readiness_report(&root, &version)?;
    let out_path = root
        .join("artifacts/release/reports")
        .join(format!("{version}-readiness-report.json"));
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    write_json(&out_path, &report)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "release_readiness_report_generate",
            "status": report["status"],
            "version": version,
            "path": repo_rel(&root, &out_path),
            "missing_required_artifacts": report["missing_required_artifacts"],
            "optional_artifacts": report["optional_artifacts"]
        }),
    )?;
    let code = if report["status"] == "ok" { 0 } else { 1 };
    Ok((rendered, code))
}

fn run_release_launch_checklist(args: ReleaseBundleBuildArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let report = generate_release_readiness_report(&root, &version)?;
    let manifest = read_json(&release_manifest_path(&root, &version))?;
    let out_path = root
        .join("artifacts/release/reports")
        .join(format!("{version}-launch-checklist.md"));
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let mut doc = String::new();
    doc.push_str("# v0.1 launch checklist\n\n");
    doc.push_str(&format!("- release version: `{version}`\n"));
    doc.push_str(&format!(
        "- readiness status: `{}`\n",
        report["status"].as_str().unwrap_or("failed")
    ));
    doc.push_str(&format!(
        "- manifest path: `{}`\n\n",
        repo_rel(&root, &release_manifest_path(&root, &version))
    ));
    doc.push_str("## Required artifacts\n\n");
    for path in report
        .get("missing_required_artifacts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
    {
        doc.push_str(&format!("- [ ] missing: `{path}`\n"));
    }
    if report
        .get("missing_required_artifacts")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| rows.is_empty())
    {
        doc.push_str("- [x] all required artifacts present\n");
    }
    doc.push_str("\n## Optional artifacts\n\n");
    for row in report
        .get("optional_artifacts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        let path = row
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let status = row
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        doc.push_str(&format!("- [{status}] `{path}`\n"));
    }
    doc.push_str("\n## Manifest summary\n\n");
    doc.push_str(&format!(
        "- git sha: `{}`\n",
        manifest
            .get("git_sha")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
    ));
    doc.push_str(&format!(
        "- chart: `{}`\n",
        manifest
            .get("chart")
            .and_then(|v| v.get("path"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
    ));
    fs::write(&out_path, doc)
        .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
    let status = report["status"].as_str().unwrap_or("failed");
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "release_launch_checklist_generate",
            "status": status,
            "version": version,
            "path": repo_rel(&root, &out_path)
        }),
    )?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

fn run_release_changelog_generate(
    args: ReleaseChangelogGenerateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let changelog_path = root.join("CHANGELOG.md");
    let mut body = fs::read_to_string(&changelog_path)
        .map_err(|err| format!("read CHANGELOG.md failed: {err}"))?;
    if !body.contains(&format!("## v{version}")) {
        let insertion = format!(
            "## v{version}\n\n### Added\n- \n\n### Changed\n- \n\n### Fixed\n- \n\n### Breaking Changes\n- none\n\n"
        );
        if let Some(pos) = body.find('\n') {
            body.insert_str(pos + 1, &format!("\n{insertion}"));
        } else {
            body.push_str(&format!("\n{insertion}"));
        }
        fs::write(&changelog_path, body)
            .map_err(|err| format!("write {} failed: {err}", changelog_path.display()))?;
    }
    let versions = extract_changelog_versions(
        &fs::read_to_string(&changelog_path)
            .map_err(|err| format!("read CHANGELOG.md failed: {err}"))?,
    );
    generate_release_docs_artifacts(&root, &version, &versions)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "kind": "release_changelog_generate",
            "status": "ok",
            "version": version,
            "path": "CHANGELOG.md"
        }),
    )?;
    Ok((rendered, 0))
}

fn run_release_changelog_validate(
    args: ReleaseChangelogValidateArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let version = args
        .version
        .unwrap_or_else(|| default_release_version(&root));
    let policy_path = root.join("configs/sources/release/version-policy.json");
    let policy = read_json(&policy_path)?;
    let required_sections = policy
        .get("changelog")
        .and_then(|v| v.get("required_sections"))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let changelog = fs::read_to_string(root.join("CHANGELOG.md"))
        .map_err(|err| format!("failed to read CHANGELOG.md: {err}"))?;
    let mut errors = Vec::<String>::new();
    let marker = format!("## v{version}");
    let Some(start) = changelog.find(&marker) else {
        errors.push(format!("missing changelog entry for version `{version}`"));
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "release_changelog_validate",
            "status": "failed",
            "version": version,
            "errors": errors
        });
        let rendered = emit_payload(args.format, args.out, &payload)?;
        return Ok((rendered, 1));
    };
    let tail = &changelog[start..];
    let next = tail.find("\n## ").unwrap_or(tail.len());
    let entry = &tail[..next];
    for section in required_sections {
        if !entry.contains(&format!("### {section}")) {
            errors.push(format!("changelog entry missing section `{section}`"));
        }
    }
    if policy
        .get("changelog")
        .and_then(|v| v.get("require_breaking_section"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
        && !entry.contains("### Breaking Changes")
    {
        errors.push("changelog entry must include breaking changes section".to_string());
    }
    if let Some(tag) = args.tag {
        let tag_version = version_from_tag(&tag);
        if tag_version != version {
            errors.push(format!(
                "release tag `{tag}` does not match changelog version `{version}`"
            ));
        }
    }
    let metadata_path = root.join("ops/release/generated/release-metadata.json");
    if metadata_path.exists() {
        let metadata = read_json(&metadata_path)?;
        let artifact_root = metadata
            .get("artifact_root")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if artifact_root != format!("artifacts/release/{version}") {
            errors.push("release docs reference incorrect artifact version".to_string());
        }
    } else {
        errors.push("release metadata page is missing".to_string());
    }
    let status = if errors.is_empty() { "ok" } else { "failed" };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "release_changelog_validate",
        "status": status,
        "version": version,
        "errors": errors
    });
    let rendered = emit_payload(args.format, args.out, &payload)?;
    Ok((rendered, if status == "ok" { 0 } else { 1 }))
}

pub(crate) fn run_release_command(
    _quiet: bool,
    command: ReleaseCommand,
) -> Result<(String, i32), String> {
    match command {
        ReleaseCommand::Plan(args) => run_release_plan(args),
        ReleaseCommand::CompatibilityCheck(args) => run_release_compatibility_check(args),
        ReleaseCommand::UpgradePlan(args) => run_release_transition_plan(args, "upgrade"),
        ReleaseCommand::RollbackPlan(args) => run_release_transition_plan(args, "rollback"),
        ReleaseCommand::Validate(args) => run_release_validate(args),
        ReleaseCommand::Tag(args) => run_release_version_check(args),
        ReleaseCommand::Notes(args) => run_release_changelog_generate(args),
        ReleaseCommand::Check(args) => run_release_check(args),
        ReleaseCommand::Rebuild { command } => match command {
            crate::cli::ReleaseRebuildCommand::Verify(args) => run_release_rebuild_verify(args),
        },
        ReleaseCommand::Reproducibility { command } => match command {
            crate::cli::ReleaseReproducibilityCommand::Report(args) => {
                run_release_reproducibility_report(args)
            }
        },
        ReleaseCommand::Version { command } => match command {
            crate::cli::ReleaseVersionCommand::Check(args) => run_release_version_check(args),
        },
        ReleaseCommand::Changelog { command } => match command {
            crate::cli::ReleaseChangelogCommand::Generate(args) => {
                run_release_changelog_generate(args)
            }
            crate::cli::ReleaseChangelogCommand::Validate(args) => {
                run_release_changelog_validate(args)
            }
        },
        ReleaseCommand::Manifest { command } => match command {
            crate::cli::ReleaseManifestCommand::Generate(args) => {
                run_release_manifest_generate(args)
            }
            crate::cli::ReleaseManifestCommand::Validate(args) => {
                run_release_manifest_validate(args)
            }
        },
        ReleaseCommand::Checksums { command } => match command {
            crate::cli::ReleaseChecksumsCommand::Generate(args) => {
                run_release_checksums_generate(args)
            }
            crate::cli::ReleaseChecksumsCommand::Verify(args) => run_release_checksums_verify(args),
        },
        ReleaseCommand::Bundle { command } => match command {
            crate::cli::ReleaseBundleCommand::Build(args) => run_release_bundle_build(args),
            crate::cli::ReleaseBundleCommand::Verify(args) => run_release_bundle_verify(args),
            crate::cli::ReleaseBundleCommand::Hash(args) => run_release_bundle_hash(args),
        },
        ReleaseCommand::ReadinessReport(args) => run_release_readiness_report(args),
        ReleaseCommand::LaunchChecklist(args) => run_release_launch_checklist(args),
        ReleaseCommand::Sign(args) => run_release_sign(args),
        ReleaseCommand::Verify(args) => run_release_verify(args),
        ReleaseCommand::Diff(args) => run_release_diff(args),
        ReleaseCommand::Packet(args) => run_release_packet(args),
        ReleaseCommand::Crates { command } => match command {
            ReleaseCratesCommand::List(args) => run_release_crates_list(args),
            ReleaseCratesCommand::ValidateMetadata(args) => {
                run_release_crates_validate_metadata(args)
            }
            ReleaseCratesCommand::ValidatePublishFlags(args) => {
                run_release_crates_validate_publish_flags(args)
            }
            ReleaseCratesCommand::DryRun(args) => run_release_crates_dry_run(args),
            ReleaseCratesCommand::PublishPlan(args) => run_release_crates_publish_plan(args),
        },
        ReleaseCommand::ApiSurface { command } => match command {
            ReleaseApiSurfaceCommand::Snapshot(args) => run_release_api_surface_snapshot(args),
        },
        ReleaseCommand::Semver { command } => match command {
            ReleaseSemverCommand::Check(args) => run_release_semver_check(args),
        },
        ReleaseCommand::Msrv { command } => match command {
            ReleaseMsrvCommand::Verify(args) => run_release_msrv_verify(args),
        },
        ReleaseCommand::Images { command } => match command {
            ReleaseImagesCommand::ValidateLabels(args) => run_release_images_validate_labels(args),
            ReleaseImagesCommand::ValidateTags(args) => run_release_images_validate_tags(args),
            ReleaseImagesCommand::ValidateBaseDigests(args) => {
                run_release_images_validate_base_digests(args)
            }
            ReleaseImagesCommand::SbomVerify(args) => run_release_images_sbom_verify(args),
            ReleaseImagesCommand::ProvenanceVerify(args) => {
                run_release_images_provenance_verify(args)
            }
            ReleaseImagesCommand::ScanVerify(args) => run_release_images_scan_verify(args),
            ReleaseImagesCommand::SmokeVerify(args) => run_release_images_smoke_verify(args),
            ReleaseImagesCommand::SizeReport(args) => run_release_images_size_report(args),
            ReleaseImagesCommand::RuntimeHardeningVerify(args) => {
                run_release_images_runtime_hardening_verify(args)
            }
            ReleaseImagesCommand::RuntimeCommandVerify(args) => {
                run_release_images_runtime_command_verify(args)
            }
            ReleaseImagesCommand::ManifestGenerate(args) => {
                run_release_images_manifest_generate(args)
            }
            ReleaseImagesCommand::ManifestVerify(args) => run_release_images_manifest_verify(args),
            ReleaseImagesCommand::ReleaseNotesCheck(args) => {
                run_release_images_release_notes_check(args)
            }
            ReleaseImagesCommand::ChangelogExtract(args) => {
                run_release_images_changelog_extract(args)
            }
            ReleaseImagesCommand::IntegrationVerify(args) => {
                run_release_images_integration_verify(args)
            }
            ReleaseImagesCommand::BuildReproducibilityCheck(args) => {
                run_release_images_build_reproducibility_check(args)
            }
            ReleaseImagesCommand::LockedDependenciesVerify(args) => {
                run_release_images_locked_dependencies_verify(args)
            }
            ReleaseImagesCommand::LockDriftVerify(args) => {
                run_release_images_lock_drift_verify(args)
            }
            ReleaseImagesCommand::ReadinessSummary(args) => {
                run_release_images_readiness_summary(args)
            }
        },
        ReleaseCommand::Ops { command } => match command {
            ReleaseOpsCommand::Package(args) => run_release_ops_package(args),
            ReleaseOpsCommand::ValidatePackage(args) => run_release_ops_validate_package(args),
            ReleaseOpsCommand::CompatibilityMatrix(args) => {
                run_release_ops_compatibility_matrix(args)
            }
            ReleaseOpsCommand::Push(args) => run_release_ops_push(args),
            ReleaseOpsCommand::DigestVerify(args) => run_release_ops_digest_verify(args),
            ReleaseOpsCommand::PullTest(args) => run_release_ops_pull_test(args),
            ReleaseOpsCommand::BundleBuild(args) => run_release_ops_bundle_build(args),
            ReleaseOpsCommand::BundleVerify(args) => run_release_ops_bundle_verify(args),
            ReleaseOpsCommand::ValuesCoverage(args) => run_release_ops_values_coverage(args),
            ReleaseOpsCommand::ProfilesVerify(args) => run_release_ops_profiles_verify(args),
            ReleaseOpsCommand::LineageGenerate(args) => run_release_ops_lineage_generate(args),
            ReleaseOpsCommand::ProvenanceVerify(args) => run_release_ops_provenance_verify(args),
            ReleaseOpsCommand::ReadinessSummary(args) => run_release_ops_readiness_summary(args),
            ReleaseOpsCommand::ScenarioEvidenceVerify(args) => {
                run_release_ops_scenario_evidence_verify(args)
            }
            ReleaseOpsCommand::PublishPlan(args) => run_release_ops_publish_plan(args),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_REPO_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    fn cleanup_release_test_repo(root: &PathBuf) {
        let _ = fs::remove_dir_all(root);
    }

    fn create_release_test_repo(changelog: &str, policy: &str) -> PathBuf {
        let stamp = TEST_REPO_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let root = std::env::temp_dir().join(format!("bijux-release-tests-{pid}-{stamp}"));
        cleanup_release_test_repo(&root);
        fs::create_dir_all(root.join("configs/sources/release"))
            .expect("create release config directory");
        fs::write(root.join("CHANGELOG.md"), changelog).expect("write changelog");
        fs::write(
            root.join("configs/sources/release/version-policy.json"),
            policy,
        )
        .expect("write policy");
        root
    }

    #[test]
    fn release_version_check_rejects_disallowed_prerelease_tag() {
        let root = create_release_test_repo(
            "# Changelog\n\n## v0.2.0-rc.1\n\n### Added\n- x\n\n### Changed\n- x\n\n### Fixed\n- x\n\n### Breaking Changes\n- none\n",
            "{\n  \"schema_version\": 1,\n  \"versioning\": {\n    \"scheme\": \"semver\",\n    \"allow_prerelease_tags\": [\"beta\"],\n    \"require_v_prefix_for_tags\": true\n  },\n  \"changelog\": {\n    \"path\": \"CHANGELOG.md\",\n    \"required_sections\": [\"Added\", \"Changed\", \"Fixed\", \"Breaking Changes\"],\n    \"require_breaking_section\": true\n  }\n}",
        );
        let args = ReleaseVersionCheckArgs {
            repo_root: Some(root.clone()),
            version: Some("0.2.0-rc.1".to_string()),
            tag: None,
            format: FormatArg::Json,
            out: None,
        };
        let (_, exit_code) = run_release_version_check(args).expect("version check should run");
        assert_eq!(exit_code, 1, "disallowed prerelease tag must fail");
        cleanup_release_test_repo(&root);
    }

    #[test]
    fn release_version_check_rejects_non_monotonic_version_progression() {
        let root = create_release_test_repo(
            "# Changelog\n\n## v0.1.0\n\n### Added\n- x\n\n### Changed\n- x\n\n### Fixed\n- x\n\n### Breaking Changes\n- none\n\n## v0.2.0\n\n### Added\n- x\n\n### Changed\n- x\n\n### Fixed\n- x\n\n### Breaking Changes\n- none\n",
            "{\n  \"schema_version\": 1,\n  \"versioning\": {\n    \"scheme\": \"semver\",\n    \"allow_prerelease_tags\": [\"rc\"],\n    \"require_v_prefix_for_tags\": true\n  },\n  \"changelog\": {\n    \"path\": \"CHANGELOG.md\",\n    \"required_sections\": [\"Added\", \"Changed\", \"Fixed\", \"Breaking Changes\"],\n    \"require_breaking_section\": true\n  }\n}",
        );
        let args = ReleaseVersionCheckArgs {
            repo_root: Some(root.clone()),
            version: Some("0.1.0".to_string()),
            tag: Some("v0.1.0".to_string()),
            format: FormatArg::Json,
            out: None,
        };
        let (_, exit_code) = run_release_version_check(args).expect("version check should run");
        assert_eq!(exit_code, 1, "version progression must be monotonic");
        cleanup_release_test_repo(&root);
    }

    #[test]
    fn release_runtime_command_verify_uses_current_cli_sources() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf();
        let (_, exit_code) = run_release_images_runtime_command_verify(ReleaseImagesValidateArgs {
            repo_root: Some(root),
            format: FormatArg::Json,
            out: None,
        })
        .expect("runtime command verify");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn release_verify_accepts_evidence_directory_path() {
        let root = std::env::temp_dir().join("bijux-release-verify-dir");
        cleanup_release_test_repo(&root);
        fs::create_dir_all(root.join("ops/release/evidence")).expect("create evidence directory");
        let tarball = resolve_release_evidence_tarball(&root, Path::new("ops/release/evidence"));
        assert_eq!(tarball, root.join("ops/release/evidence/bundle.tar"));
        cleanup_release_test_repo(&root);
    }

    #[test]
    fn release_verify_keeps_explicit_tarball_path() {
        let root = std::env::temp_dir().join("bijux-release-verify-tarball");
        cleanup_release_test_repo(&root);
        fs::create_dir_all(root.join("ops/release/evidence")).expect("create evidence directory");
        let tarball =
            resolve_release_evidence_tarball(&root, Path::new("ops/release/evidence/custom.tar"));
        assert_eq!(tarball, root.join("ops/release/evidence/custom.tar"));
        cleanup_release_test_repo(&root);
    }

    #[test]
    fn release_tarball_members_include_ops_evidence_artifacts() {
        let root = std::env::temp_dir().join("bijux-release-tarball-members");
        cleanup_release_test_repo(&root);
        let evidence_dir = root.join("ops/release/evidence");
        fs::create_dir_all(&evidence_dir).expect("create evidence dir");
        fs::write(evidence_dir.join("manifest.json"), "{}").expect("write manifest");
        let rel = "artifacts/ops/evidence/ops_run/install-evidence.json";
        let abs = root.join(rel);
        fs::create_dir_all(abs.parent().expect("parent")).expect("create parent");
        fs::write(&abs, "{}").expect("write artifact");
        let manifest = serde_json::json!({
            "ops_evidence": {
                "install_evidence": {
                    "path": rel
                },
                "redaction_secret_keys": ["token"]
            }
        });

        let members = collect_tarball_members(&root, &manifest).expect("collect members");

        assert!(members.contains(&"ops/release/evidence/manifest.json".to_string()));
        assert!(members.contains(&rel.to_string()));
        cleanup_release_test_repo(&root);
    }

    #[test]
    fn publishable_package_surface_requires_readme_links_to_homepage_and_docs() {
        let package = toml::toml! {
            homepage = "https://bijux.github.io/bijux-atlas/"
            documentation = "https://docs.rs/bijux-atlas/latest/bijux_atlas/"
            repository = "https://github.com/bijux/bijux-atlas.git"
        };
        let errors =
            validate_publishable_package_surface("bijux-atlas", &package, None, "# crate\n");

        assert!(errors
            .iter()
            .any(|err| err.contains("README must link to package.homepage")));
        assert!(errors
            .iter()
            .any(|err| err.contains("README must link to package.documentation")));
    }

    #[test]
    fn publishable_package_surface_rejects_parent_relative_readme_links() {
        let package = toml::toml! {
            homepage = "https://bijux.github.io/bijux-atlas/"
            documentation = "https://docs.rs/bijux-atlas/latest/bijux_atlas/"
            repository = "https://github.com/bijux/bijux-atlas.git"
        };
        let readme = r#"
[Docs](https://bijux.github.io/bijux-atlas/)
[API](https://docs.rs/bijux-atlas/latest/bijux_atlas/)
[Control Plane](../bijux-dev-atlas/README.md)
"#;
        let errors = validate_publishable_package_surface("bijux-atlas", &package, None, readme);

        assert!(errors
            .iter()
            .any(|err| err.contains("parent-relative link `../bijux-dev-atlas/README.md`")));
    }

    #[test]
    fn resolve_package_field_reads_workspace_metadata() {
        let package = toml::toml! {
            repository = { workspace = true }
        };
        let workspace_package = toml::toml! {
            repository = "https://github.com/bijux/bijux-atlas.git"
        };

        let resolved = resolve_package_field(&package, Some(&workspace_package), "repository");

        assert_eq!(
            resolved.as_deref(),
            Some("https://github.com/bijux/bijux-atlas.git")
        );
    }

    #[test]
    fn markdown_link_targets_extracts_inline_links() {
        let markdown = r#"
[Docs](https://bijux.github.io/bijux-atlas/)
text [API](https://docs.rs/bijux-atlas/latest/bijux_atlas/) end
[Control Plane](../bijux-dev-atlas/README.md)
"#;

        let targets = markdown_link_targets(markdown);

        assert_eq!(
            targets,
            vec![
                "https://bijux.github.io/bijux-atlas/",
                "https://docs.rs/bijux-atlas/latest/bijux_atlas/",
                "../bijux-dev-atlas/README.md"
            ]
        );
    }
}
