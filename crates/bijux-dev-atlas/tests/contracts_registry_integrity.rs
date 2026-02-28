// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;

use sha2::{Digest, Sha256};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

fn contract_rows_for_domain(domain: &str) -> Vec<bijux_dev_atlas::contracts::RegistrySnapshotRow> {
    let root = repo_root();
    let contracts = match domain {
        "docker" => bijux_dev_atlas::contracts::docker::contracts(&root).expect("docker contracts"),
        "make" => bijux_dev_atlas::contracts::make::contracts(&root).expect("make contracts"),
        "ops" => bijux_dev_atlas::contracts::ops::contracts(&root).expect("ops contracts"),
        _ => panic!("unsupported domain"),
    };
    bijux_dev_atlas::contracts::registry_snapshot(domain, &contracts)
}

fn contracts_list_json(domain: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(["contracts", domain, "--list", "--format", "json"])
        .output()
        .expect("contracts list");
    assert!(output.status.success(), "list failed for {domain}");
    serde_json::from_slice(&output.stdout).expect("json list")
}

fn explain_json(domain: &str, contract_id: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args([
            "contracts",
            domain,
            "--explain",
            contract_id,
            "--format",
            "json",
        ])
        .output()
        .expect("contracts explain");
    assert!(
        output.status.success(),
        "explain failed for {domain}:{contract_id}"
    );
    serde_json::from_slice(&output.stdout).expect("json explain")
}

fn output_sha256(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("contracts command");
    assert!(output.status.success(), "command failed for {:?}", args);
    let mut hasher = Sha256::new();
    hasher.update(&output.stdout);
    format!("{:x}", hasher.finalize())
}

#[test]
fn human_output_hashes_are_stable_for_static_contract_runs() {
    let cases = [
        (
            vec![
                "contracts",
                "docker",
                "--mode",
                "static",
                "--format",
                "human",
            ],
            "284239285c8729919449e19184102e9d03a9c3f65dd44058c7d94c695d1f209d",
        ),
        (
            vec!["contracts", "ops", "--mode", "static", "--format", "human"],
            "ed573cb9d7ffe27fdabe8df50c51dbba71667a2c745d6464a80025f77ef4b8ac",
        ),
        (
            vec!["contracts", "make", "--mode", "static", "--format", "human"],
            "5be1c13dde525172b0069e7682d414fa40621474cadb35a4be59f366b80ab5bd",
        ),
        (
            vec!["contracts", "all", "--mode", "static", "--format", "human"],
            "ec473d2cfe8bfa50e61e04c60c2f6e7227838c8095beb6695d5bb0b821aee726",
        ),
    ];
    for (args, expected) in cases {
        assert_eq!(
            output_sha256(&args),
            expected,
            "snapshot drift for {:?}",
            args
        );
    }
}

#[test]
fn registry_list_matches_registry_snapshot_and_explain_output() {
    for domain in ["docker", "make", "ops"] {
        let rows = contract_rows_for_domain(domain);
        let listed = contracts_list_json(domain);
        let list_rows = listed["contracts"].as_array().expect("list rows");
        assert_eq!(
            rows.len(),
            list_rows.len(),
            "hidden contract detected in {domain}"
        );

        let listed_map = list_rows
            .iter()
            .map(|row| {
                let id = row["id"].as_str().expect("id").to_string();
                let tests = row["tests"]
                    .as_array()
                    .expect("tests")
                    .iter()
                    .map(|test| test["test_id"].as_str().expect("test_id").to_string())
                    .collect::<Vec<_>>();
                (id, tests)
            })
            .collect::<BTreeMap<_, _>>();

        for row in rows {
            let listed_tests = listed_map.get(&row.id).expect("row in list");
            assert_eq!(
                listed_tests, &row.test_ids,
                "list drift for {domain}:{}",
                row.id
            );

            let explain = explain_json(domain, &row.id);
            let mut explain_tests = explain["tests"]
                .as_array()
                .expect("explain tests")
                .iter()
                .map(|test| test["test_id"].as_str().expect("test_id").to_string())
                .collect::<Vec<_>>();
            explain_tests.sort();
            assert_eq!(
                explain_tests, row.test_ids,
                "explain drift for {domain}:{}",
                row.id
            );
        }
    }
}

#[test]
fn every_contract_test_belongs_to_exactly_one_contract() {
    let mut owners = BTreeMap::<String, Vec<String>>::new();
    for domain in ["docker", "make", "ops"] {
        for row in contract_rows_for_domain(domain) {
            for test_id in row.test_ids {
                owners
                    .entry(test_id)
                    .or_default()
                    .push(format!("{domain}:{}", row.id));
            }
        }
    }
    for (test_id, rows) in owners {
        assert_eq!(
            rows.len(),
            1,
            "orphan or duplicate test ownership for {test_id}: {rows:?}"
        );
    }
}

#[test]
fn contract_debt_registry_covers_all_contract_domains() {
    let root = repo_root();
    let text =
        std::fs::read_to_string(root.join("ops/inventory/contract-debt.json")).expect("debt file");
    let payload: serde_json::Value = serde_json::from_str(&text).expect("debt json");
    let domains = payload["items"]
        .as_array()
        .expect("items")
        .iter()
        .filter_map(|item| item["domain"].as_str())
        .collect::<BTreeSet<_>>();
    assert!(domains.contains("docker"));
    assert!(domains.contains("make"));
    assert!(domains.contains("ops"));
}
