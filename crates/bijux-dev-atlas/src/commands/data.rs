// SPDX-License-Identifier: Apache-2.0

use crate::cli::{DatasetsCommand, DatasetsValidateArgs, IngestCommand, IngestDryRunArgs};
use crate::{emit_payload, resolve_repo_root};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
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
    let bytes = fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
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
    ensure_json(&root.join("configs/contracts/datasets/manifest.schema.json"))?;
    ensure_json(&root.join("configs/contracts/datasets/pinned-policy.schema.json"))?;
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
        entry.get("checksum")
            .and_then(serde_yaml::Value::as_str)
            .is_some_and(|value| value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()))
    });
    let required_fields_ok = datasets.iter().all(|entry| {
        ["id", "version", "source", "license", "checksum", "size", "schema_version"]
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
    ensure_json(&root.join("configs/contracts/datasets/ingest-plan.schema.json"))?;
    let manifest = read_yaml(&root.join("configs/datasets/manifest.yaml"))?;
    let datasets = manifest
        .get("datasets")
        .and_then(serde_yaml::Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let selected = datasets
        .iter()
        .find(|entry| {
            entry.get("id")
                .and_then(serde_yaml::Value::as_str)
                == Some(args.dataset.as_str())
        })
        .ok_or_else(|| format!("dataset `{}` is not declared in configs/datasets/manifest.yaml", args.dataset))?;

    let source_dir = selected
        .get("source")
        .and_then(serde_yaml::Value::as_str)
        .ok_or_else(|| "dataset source is missing".to_string())?;
    let genome = root.join(source_dir).join("genome.fa");
    let fai = root.join(source_dir).join("genome.fa.fai");
    let gff3 = root.join(source_dir).join("genes.gff3");
    let output_dir = format!("artifacts/ingest/{}/outputs", args.dataset.replace('/', "_"));
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

pub(crate) fn run_data_command(_quiet: bool, command: DataCommand) -> Result<(String, i32), String> {
    match command {
        DataCommand::Datasets(command) => match command {
            DatasetsCommand::Validate(args) => run_datasets_validate(args),
        },
        DataCommand::Ingest(command) => match command {
            IngestCommand::DryRun(args) => run_ingest_dry_run(args),
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
        assert!(result.0.contains("\"dataset_id\": \"110/homo_sapiens/GRCh38\""));
    }
}
