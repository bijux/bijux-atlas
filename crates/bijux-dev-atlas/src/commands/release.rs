// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    FormatArg, ReleaseBundleBuildArgs, ReleaseBundleHashArgs, ReleaseBundleVerifyArgs,
    ReleaseCheckArgs, ReleaseCommand, ReleaseDiffArgs, ReleaseManifestGenerateArgs,
    ReleaseManifestValidateArgs, ReleasePacketArgs, ReleaseSignArgs, ReleaseVerifyArgs,
};
use crate::{emit_payload, resolve_repo_root};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let evidence_root = root.join("release/evidence");
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

    let ok = validate_out.status.success() && readiness_out.status.success();
    let payload = serde_json::json!({
        "schema_version": 1,
        "status": if ok { "ok" } else { "failed" },
        "text": if ok { "release check passed" } else { "release check failed" },
        "validate": validate_payload,
        "ops_validate": readiness_payload
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

fn repo_rel<'a>(root: &'a Path, path: &'a Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn run_release_sign(args: ReleaseSignArgs) -> Result<(String, i32), String> {
    let repo_root_arg = args.repo_root.clone();
    let root = resolve_repo_root(repo_root_arg.clone())?;
    ensure_json(&root.join("configs/contracts/release/signing-policy.schema.json"))?;
    ensure_json(&root.join("configs/contracts/release/checksum-list.schema.json"))?;
    ensure_json(&root.join("configs/contracts/release/release-sign.schema.json"))?;
    ensure_json(&root.join("configs/contracts/release/provenance.schema.json"))?;

    let evidence_dir = if args.evidence.is_absolute() {
        args.evidence
    } else {
        root.join(args.evidence)
    };
    let policy_path = root.join("release/signing/policy.yaml");
    let checksums_path = root.join("release/signing/checksums.json");
    let sign_report_path = root.join("release/signing/release-sign.json");
    let provenance_path = root.join("release/provenance.json");
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
        "toolchain_inventory": "configs/rust/toolchain.json",
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
    let verify_report_path = root.join("release/signing/release-verify.json");
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
    let tarball_path = root.join("release/evidence/bundle.tar");
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
    ensure_json(&root.join("configs/contracts/release/checksum-list.schema.json"))?;
    ensure_json(&root.join("configs/contracts/release/release-sign.schema.json"))?;
    ensure_json(&root.join("configs/contracts/release/release-verify.schema.json"))?;
    ensure_json(&root.join("configs/contracts/release/provenance.schema.json"))?;

    let tarball = if args.evidence.is_absolute() {
        args.evidence
    } else {
        root.join(args.evidence)
    };
    let checksums_path = root.join("release/signing/checksums.json");
    let sign_report_path = root.join("release/signing/release-sign.json");
    let verify_report_path = root.join("release/signing/release-verify.json");
    let provenance_path = root.join("release/provenance.json");
    let policy_path = root.join("release/signing/policy.yaml");

    let checksums = read_json(&checksums_path)?;
    let sign_report = read_json(&sign_report_path)?;
    let provenance = read_json(&provenance_path)?;
    let policy = read_yaml(&policy_path)?;
    let manifest = read_json(&root.join("release/evidence/manifest.json"))?;
    let manifest_schema = read_json(&root.join("release/evidence/manifest.schema.json"))?;

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
        "release/signing/checksums.json",
        "release/signing/release-sign.json",
        "release/provenance.json",
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
        "release/evidence/manifest.json",
        "release/signing/checksums.json",
        "release/signing/release-sign.json",
        "release/signing/release-verify.json",
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
    let rel_sign_004 = rel_sign_001
        && rel_sign_002
        && rel_sign_003
        && rel_sign_005
        && rel_sign_006
        && rel_prov_001
        && rel_man_001
        && rel_tar_001
        && evidence_ok;
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
            "REL-PROV-001": rel_prov_001
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

fn classify_release_diff(path: &str) -> &'static str {
    if path.starts_with("release/signing/") || path == "release/provenance.json" {
        "signing"
    } else if path.starts_with("release/evidence/sboms/") {
        "sbom"
    } else if path.starts_with("release/evidence/packages/") {
        "artifact"
    } else if path.starts_with("release/evidence/") {
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
    ensure_json(&root.join("configs/contracts/release/packet-list.schema.json"))?;

    let evidence_dir = if args.evidence.is_absolute() {
        args.evidence
    } else {
        root.join(args.evidence)
    };
    let manifest_path = evidence_dir.join("manifest.json");
    let manifest = read_json(&manifest_path)?;
    let packet_dir = root.join("release/packet");
    let packet_path = packet_dir.join("packet.json");

    let required = [
        "release/evidence/manifest.json",
        "release/evidence/identity.json",
        "release/evidence/bundle.tar",
        "release/signing/checksums.json",
        "release/signing/release-sign.json",
        "release/signing/release-verify.json",
        "release/provenance.json",
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
                .is_some_and(|path| path.starts_with("release/evidence/sboms/"))
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
    let chart_yaml = root.join("ops/k8s/charts/bijux-atlas/Chart.yaml");
    if let Ok(value) = read_yaml(&chart_yaml) {
        if let Some(version) = value.get("version").and_then(serde_yaml::Value::as_str) {
            if !version.trim().is_empty() {
                return version.to_string();
            }
        }
    }
    "0.0.0".to_string()
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
    read_json(&root.join("release/evidence/manifest.json"))
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
    let build_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
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
        "control_plane_version": env!("CARGO_PKG_VERSION"),
        "ops_profiles_validated": ops_profiles,
        "docs_build_hash": docs_hash,
        "sbom_digests": sbom_refs,
        "container_image_digests": image_digests,
        "chart": {
            "version": version,
            "path": chart_path,
            "sha256": chart_digest
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
    ensure_json(&root.join("configs/contracts/release/release-manifest.schema.json"))?;
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

    let provenance_src = root.join("release/provenance.json");
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
            if !required.contains(name.as_str()) && name != "bundle.sha256" {
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

pub(crate) fn run_release_command(
    _quiet: bool,
    command: ReleaseCommand,
) -> Result<(String, i32), String> {
    match command {
        ReleaseCommand::Check(args) => run_release_check(args),
        ReleaseCommand::Manifest { command } => match command {
            crate::cli::ReleaseManifestCommand::Generate(args) => {
                run_release_manifest_generate(args)
            }
            crate::cli::ReleaseManifestCommand::Validate(args) => {
                run_release_manifest_validate(args)
            }
        },
        ReleaseCommand::Bundle { command } => match command {
            crate::cli::ReleaseBundleCommand::Build(args) => run_release_bundle_build(args),
            crate::cli::ReleaseBundleCommand::Verify(args) => run_release_bundle_verify(args),
            crate::cli::ReleaseBundleCommand::Hash(args) => run_release_bundle_hash(args),
        },
        ReleaseCommand::Sign(args) => run_release_sign(args),
        ReleaseCommand::Verify(args) => run_release_verify(args),
        ReleaseCommand::Diff(args) => run_release_diff(args),
        ReleaseCommand::Packet(args) => run_release_packet(args),
    }
}
