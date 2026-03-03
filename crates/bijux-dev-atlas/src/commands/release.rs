// SPDX-License-Identifier: Apache-2.0

use crate::cli::{FormatArg, ReleaseCheckArgs, ReleaseCommand, ReleaseSignArgs, ReleaseVerifyArgs};
use crate::{emit_payload, resolve_repo_root};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
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
        "checksums_path": repo_rel(&root, &checksums_path),
        "provenance_path": repo_rel(&root, &provenance_path),
        "signed_items": checksum_list["items"].clone(),
        "verification_command": verification_command,
        "mechanism": mechanism,
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
                "checksums_path": repo_rel(&root, &checksums_path),
                "provenance_path": repo_rel(&root, &provenance_path),
                "contracts": {
                    "REL-SIGN-001": sign_report["status"] == "ok",
                    "REL-SIGN-002": sign_report["status"] == "ok",
                    "REL-SIGN-003": sign_report["status"] == "ok",
                    "REL-PROV-001": true
                },
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
    let rel_prov_001 = provenance
        .get("schema_version")
        .and_then(serde_json::Value::as_i64)
        == Some(1)
        && provenance
            .get("git_sha")
            .and_then(serde_json::Value::as_str)
            .is_some();

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
    let rel_sign_004 = rel_sign_001 && rel_sign_002 && rel_sign_003 && rel_prov_001 && evidence_ok;
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
    if !rel_prov_001 {
        errors.push("provenance file is missing required fields".to_string());
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

pub(crate) fn run_release_command(
    _quiet: bool,
    command: ReleaseCommand,
) -> Result<(String, i32), String> {
    match command {
        ReleaseCommand::Check(args) => run_release_check(args),
        ReleaseCommand::Sign(args) => run_release_sign(args),
        ReleaseCommand::Verify(args) => run_release_verify(args),
    }
}
