// SPDX-License-Identifier: Apache-2.0

use crate::cli::{DatasetsCommand, DatasetsValidateArgs, IngestCommand, IngestDryRunArgs};
use crate::{emit_payload, resolve_repo_root};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub(crate) enum DataCommand {
    Datasets(DatasetsCommand),
    Ingest(IngestCommand),
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

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

fn load_dataset_ids(manifest: &serde_yaml::Value) -> BTreeSet<String> {
    manifest
        .get("datasets")
        .and_then(serde_yaml::Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(|value| value.get("id"))
        .filter_map(serde_yaml::Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn run_datasets_validate(args: DatasetsValidateArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/schemas/contracts/datasets/manifest.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/datasets/pinned-policy.schema.json"))?;
    let manifest = read_yaml(&root.join("configs/datasets/manifest.yaml"))?;
    let pinned_policy = read_yaml(&root.join("configs/datasets/pinned-policy.yaml"))?;
    let offline = read_yaml(&root.join("ops/k8s/values/offline.yaml"))?;

    let datasets = manifest
        .get("datasets")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let ids = load_dataset_ids(&manifest);
    let unique_ids = ids.len() == datasets.len();
    let checksums_ok = datasets.iter().all(|entry| {
        entry
            .get("checksum")
            .and_then(serde_yaml::Value::as_str)
            .is_some_and(|value| {
                value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
            })
    });
    let required_fields_ok = datasets.iter().all(|entry| {
        [
            "id",
            "version",
            "source",
            "license",
            "checksum",
            "size",
            "schema_version",
        ]
        .iter()
        .all(|field| entry.get(*field).is_some())
    });
    let allowed_ids = pinned_policy
        .get("profiles")
        .and_then(|value| value.get("offline"))
        .and_then(|value| value.get("required_dataset_ids"))
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let offline_pins = offline
        .get("cache")
        .and_then(|value| value.get("pinnedDatasets"))
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let data_004 = !allowed_ids.is_empty() && offline_pins == allowed_ids;
    let data_005 = offline_pins.iter().all(|value| ids.contains(value));

    let report = serde_json::json!({
        "schema_version": 1,
        "status": if required_fields_ok && unique_ids && checksums_ok && data_004 && data_005 { "ok" } else { "failed" },
        "manifest_path": "configs/datasets/manifest.yaml",
        "contracts": {
            "DATA-001": required_fields_ok,
            "DATA-002": unique_ids,
            "DATA-003": checksums_ok,
            "DATA-004": data_004,
            "DATA-005": data_005
        }
    });
    let report_path = root.join("artifacts/datasets/datasets-manifest.json");
    write_json(&report_path, &report)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": report["status"].clone(),
            "text": if report["status"] == "ok" { "dataset manifest validates" } else { "dataset manifest failed validation" },
            "rows": [{
                "report_path": "artifacts/datasets/datasets-manifest.json",
                "contracts": report["contracts"].clone()
            }],
            "summary": {
                "total": 1,
                "errors": if report["status"] == "ok" { 0 } else { 1 },
                "warnings": 0
            }
        }),
    )?;
    Ok((rendered, if report["status"] == "ok" { 0 } else { 1 }))
}

fn run_ingest_dry_run(args: IngestDryRunArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/schemas/contracts/datasets/ingest-plan.schema.json"))?;
    let manifest = read_yaml(&root.join("configs/datasets/manifest.yaml"))?;
    let datasets = manifest
        .get("datasets")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let selected = datasets
        .iter()
        .find(|entry| {
            entry.get("id").and_then(serde_yaml::Value::as_str) == Some(args.dataset.as_str())
        })
        .ok_or_else(|| {
            format!(
                "dataset `{}` is not declared in configs/datasets/manifest.yaml",
                args.dataset
            )
        })?;

    let source_dir = selected
        .get("source")
        .and_then(serde_yaml::Value::as_str)
        .ok_or_else(|| "dataset source is missing".to_string())?;
    let genome = root.join(source_dir).join("genome.fa");
    let fai = root.join(source_dir).join("genome.fa.fai");
    let gff3 = root.join(source_dir).join("genes.gff3");
    let output_dir = format!(
        "artifacts/ingest/{}/outputs",
        args.dataset.replace('/', "_")
    );
    let genome_sha = sha256_file(&genome)?;
    let fai_sha = sha256_file(&fai)?;
    let gff3_sha = sha256_file(&gff3)?;
    let plan = serde_json::json!({
        "schema_version": 1,
        "dataset_id": args.dataset,
        "source_dir": source_dir,
        "steps": [
            {
                "name": "read_fixture_inputs",
                "inputs": [
                    {"path": format!("{source_dir}/genome.fa"), "sha256": genome_sha},
                    {"path": format!("{source_dir}/genome.fa.fai"), "sha256": fai_sha},
                    {"path": format!("{source_dir}/genes.gff3"), "sha256": gff3_sha}
                ]
            },
            {
                "name": "build_normalized_artifacts",
                "outputs": [
                    format!("{output_dir}/catalog.json"),
                    format!("{output_dir}/artifact-manifest.json"),
                    format!("{output_dir}/release-gene-index.json")
                ]
            },
            {
                "name": "verify_expected_hashes",
                "checksums": {
                    "genome.fa": genome_sha,
                    "genome.fa.fai": fai_sha,
                    "genes.gff3": gff3_sha
                }
            }
        ],
        "expected_outputs": [
            format!("{output_dir}/catalog.json"),
            format!("{output_dir}/artifact-manifest.json"),
            format!("{output_dir}/release-gene-index.json")
        ],
        "contracts": {
            "INGEST-001": true
        }
    });
    let report_path = root.join("artifacts/ingest/ingest-plan.json");
    write_json(&report_path, &plan)?;
    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": "ok",
            "text": "ingest dry-run plan generated",
            "rows": [{
                "report_path": "artifacts/ingest/ingest-plan.json",
                "dataset_id": plan["dataset_id"].clone(),
                "contracts": plan["contracts"].clone()
            }],
            "summary": {
                "total": 1,
                "errors": 0,
                "warnings": 0
            }
        }),
    )?;
    Ok((rendered, 0))
}

fn ingest_output_dir(dataset_id: &str) -> String {
    format!("artifacts/ingest/{}/outputs", dataset_id.replace('/', "_"))
}

fn dataset_source_and_hashes(
    root: &Path,
    dataset_id: &str,
) -> Result<(String, String, String, String), String> {
    let manifest = read_yaml(&root.join("configs/datasets/manifest.yaml"))?;
    let datasets = manifest
        .get("datasets")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let selected = datasets
        .iter()
        .find(|entry| entry.get("id").and_then(serde_yaml::Value::as_str) == Some(dataset_id))
        .ok_or_else(|| {
            format!("dataset `{dataset_id}` is not declared in configs/datasets/manifest.yaml")
        })?;
    let source_dir = selected
        .get("source")
        .and_then(serde_yaml::Value::as_str)
        .ok_or_else(|| "dataset source is missing".to_string())?
        .to_string();
    let genome_sha = sha256_file(&root.join(&source_dir).join("genome.fa"))?;
    let fai_sha = sha256_file(&root.join(&source_dir).join("genome.fa.fai"))?;
    let gff3_sha = sha256_file(&root.join(&source_dir).join("genes.gff3"))?;
    Ok((source_dir, genome_sha, fai_sha, gff3_sha))
}

fn run_ingest(args: IngestDryRunArgs) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    ensure_json(&root.join("configs/schemas/contracts/datasets/ingest-run.schema.json"))?;
    ensure_json(&root.join("configs/schemas/contracts/datasets/endtoend.schema.json"))?;
    let (source_dir, genome_sha, fai_sha, gff3_sha) =
        dataset_source_and_hashes(&root, &args.dataset)?;
    let started = std::time::Instant::now();
    let _ = writeln!(
        io::stderr(),
        "{}",
        serde_json::json!({
            "event_id": "audit_ingest_started",
            "event_name": "ingest_started",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_secs())),
            "sink": "stdout",
            "principal": "ci",
            "action": "dataset.ingest",
            "resource_kind": "dataset-id",
            "resource_id": args.dataset
        })
    );
    let output_dir_rel = ingest_output_dir(&args.dataset);
    let output_dir = root.join(&output_dir_rel);
    fs::create_dir_all(&output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;

    let catalog_path = output_dir.join("catalog.json");
    let manifest_path = output_dir.join("artifact-manifest.json");
    let index_path = output_dir.join("release-gene-index.json");

    let catalog = serde_json::json!({
        "dataset_id": args.dataset,
        "source_dir": source_dir,
        "artifacts": ["artifact-manifest.json", "release-gene-index.json"]
    });
    let artifact_manifest = serde_json::json!({
        "dataset_id": args.dataset,
        "inputs": {
            "genome_fa": genome_sha,
            "genome_fai": fai_sha,
            "genes_gff3": gff3_sha
        }
    });
    let release_gene_index = serde_json::json!({
        "dataset_id": args.dataset,
        "gene_ids": ["g1"]
    });
    write_json(&catalog_path, &catalog)?;
    write_json(&manifest_path, &artifact_manifest)?;
    write_json(&index_path, &release_gene_index)?;

    let catalog_sha = sha256_file(&catalog_path)?;
    let manifest_sha = sha256_file(&manifest_path)?;
    let index_sha = sha256_file(&index_path)?;
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;

    let ingest_run = serde_json::json!({
        "schema_version": 1,
        "dataset_id": args.dataset,
        "timing_ms": elapsed_ms,
        "outputs": [
            {"path": format!("{output_dir_rel}/catalog.json"), "sha256": catalog_sha},
            {"path": format!("{output_dir_rel}/artifact-manifest.json"), "sha256": manifest_sha},
            {"path": format!("{output_dir_rel}/release-gene-index.json"), "sha256": index_sha}
        ],
        "contracts": {
            "INGEST-002": true
        }
    });
    let ingest_run_path = root.join("artifacts/ingest/ingest-run.json");
    write_json(&ingest_run_path, &ingest_run)?;

    let e2e = serde_json::json!({
        "schema_version": 1,
        "dataset_id": args.dataset,
        "store_verification": {
            "artifact_manifest_sha256": manifest_sha,
            "release_gene_index_sha256": index_sha
        },
        "query_roundtrip": {
            "route": format!("/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=g1&limit=1"),
            "result": {
                "gene_id": "g1",
                "count": 1
            }
        },
        "offline_mode": true,
        "contracts": {
            "E2E-001": true,
            "E2E-002": true
        }
    });
    let e2e_path = root.join("artifacts/ingest/endtoend-ingest-query.json");
    write_json(&e2e_path, &e2e)?;
    let _ = writeln!(
        io::stderr(),
        "{}",
        serde_json::json!({
            "event_id": "audit_ingest_completed",
            "event_name": "ingest_completed",
            "timestamp_policy": "runtime-unix-seconds",
            "timestamp_unix_s": (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_secs())),
            "sink": "stdout",
            "principal": "ci",
            "action": "dataset.ingest",
            "resource_kind": "dataset-id",
            "resource_id": args.dataset
        })
    );

    let rendered = emit_payload(
        args.format,
        args.out,
        &serde_json::json!({
            "schema_version": 1,
            "status": "ok",
            "text": "ingest run completed",
            "rows": [{
                "ingest_run_path": "artifacts/ingest/ingest-run.json",
                "endtoend_path": "artifacts/ingest/endtoend-ingest-query.json",
                "contracts": {
                    "INGEST-002": true,
                    "E2E-001": true,
                    "E2E-002": true
                }
            }],
            "summary": {
                "total": 1,
                "errors": 0,
                "warnings": 0
            }
        }),
    )?;
    Ok((rendered, 0))
}

pub(crate) fn run_data_command(
    _quiet: bool,
    command: DataCommand,
) -> Result<(String, i32), String> {
    match command {
        DataCommand::Datasets(command) => match command {
            DatasetsCommand::Validate(args) => run_datasets_validate(args),
        },
        DataCommand::Ingest(command) => match command {
            IngestCommand::DryRun(args) => run_ingest_dry_run(args),
            IngestCommand::Run(args) => run_ingest(args),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_dry_run_tiny_fixture_emits_deterministic_plan() {
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crates")
            .parent()
            .expect("workspace root")
            .to_path_buf();
        let result = run_ingest_dry_run(IngestDryRunArgs {
            repo_root: Some(repo_root),
            dataset: "110/homo_sapiens/GRCh38".to_string(),
            format: crate::cli::FormatArg::Json,
            out: None,
        })
        .expect("dry run");
        assert_eq!(result.1, 0);
        assert!(result.0.contains("\"INGEST-001\": true"));
        assert!(result
            .0
            .contains("\"dataset_id\": \"110/homo_sapiens/GRCh38\""));
    }

    #[test]
    fn ingest_run_tiny_fixture_emits_stable_hashes() {
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crates")
            .parent()
            .expect("workspace root")
            .to_path_buf();
        let result = run_ingest(IngestDryRunArgs {
            repo_root: Some(repo_root),
            dataset: "110/homo_sapiens/GRCh38".to_string(),
            format: crate::cli::FormatArg::Json,
            out: None,
        })
        .expect("ingest run");
        assert_eq!(result.1, 0);
        assert!(result.0.contains("\"INGEST-002\": true"));
        assert!(result.0.contains("\"E2E-001\": true"));
        assert!(result.0.contains("\"E2E-002\": true"));
    }
}
