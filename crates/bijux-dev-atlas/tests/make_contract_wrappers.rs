// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn canonical_make_contract_targets_delegate_to_contract_runner() {
    let makefile =
        fs::read_to_string(repo_root().join("make/contracts.mk")).expect("read make/contracts.mk");
    assert!(makefile.contains("contract: _contracts_guard"));
    assert!(makefile.contains("contract-effect: _contracts_guard"));
    assert!(makefile.contains("contract-all: _contracts_guard"));
    assert!(makefile.contains("contract-list: _contracts_guard"));
    assert!(makefile.contains("contract-report: _contracts_guard"));
    assert!(makefile.contains(
        "$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode static"
    ));
    assert!(makefile.contains(
        "$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode effect --effects-policy allow"
    ));
    assert!(makefile.contains(
        "$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow"
    ));
    assert!(makefile.contains("$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract list"));
    assert!(makefile.contains("$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract report --last --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"));
}

#[test]
fn deprecated_make_contract_targets_warn_and_delegate() {
    let makefile =
        fs::read_to_string(repo_root().join("make/contracts.mk")).expect("read make/contracts.mk");
    assert!(makefile.contains("contracts: ## Deprecated alias for make contract"));
    assert!(makefile.contains("contracts-effect: ## Deprecated alias for make contract-effect"));
    assert!(makefile.contains("contracts-all: ## Deprecated alias for make contract-all"));
    assert!(makefile.contains("deprecated: use \\`make contract\\`"));
    assert!(makefile.contains("deprecated: use \\`make contract-effect\\`"));
    assert!(
        makefile.contains("deprecated: use \\`make contract-all\\`")
            || makefile.contains(
                "contracts-all: ## Deprecated alias for make contract-all\n\t@$(DEV_ATLAS) --output-format $(GLOBAL_OUTPUT_FORMAT) contract run --mode all --effects-policy allow --jobs $(JOBS) $(CONTRACT_FAIL_FAST_FLAG) $(CONTRACT_NO_ANSI_FLAG) --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"
            ),
        "contracts-all must either print the deprecation hint or directly delegate to contract-all equivalent command"
    );
}

#[test]
fn canonical_make_contract_targets_do_not_parse_output() {
    let makefile =
        fs::read_to_string(repo_root().join("make/contracts.mk")).expect("read make/contracts.mk");
    for forbidden in ["grep", "jq", "awk", "sed"] {
        let needle = format!("contract: _contracts_guard\n\t@{forbidden}");
        assert!(
            !makefile.contains(&needle),
            "canonical contract target must not shell-parse output with {forbidden}"
        );
        let needle = format!("contract-effect: _contracts_guard\n\t@{forbidden}");
        assert!(
            !makefile.contains(&needle),
            "canonical contract-effect target must not shell-parse output with {forbidden}"
        );
        let needle = format!("contract-all: _contracts_guard\n\t@{forbidden}");
        assert!(
            !makefile.contains(&needle),
            "canonical contract-all target must not shell-parse output with {forbidden}"
        );
        let needle = format!("contract-list: _contracts_guard\n\t@{forbidden}");
        assert!(
            !makefile.contains(&needle),
            "canonical contract-list target must not shell-parse output with {forbidden}"
        );
    }
}

#[test]
fn make_wrappers_verify_command_runs_contract_verification() {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["make", "wrappers", "verify", "--format", "json"])
        .output()
        .expect("run make wrappers verify");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json payload");
    assert_eq!(payload["kind"], "make_wrappers_verify");
    let status = payload["status"].as_str().unwrap_or("");
    assert!(
        status == "ok" || status == "failed",
        "make wrappers verify must report stable status, got {status}"
    );
    if output.status.success() {
        assert_eq!(status, "ok");
    } else {
        assert_eq!(status, "failed");
    }
}

#[test]
fn make_target_list_snapshot_matches_curated_surface_and_is_sorted() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["make", "list", "--format", "json"])
        .output()
        .expect("run make list");
    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json payload");
    let observed = payload["public_targets"]
        .as_array()
        .expect("public_targets array")
        .iter()
        .filter_map(|v| v.as_str())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let target_list_text =
        fs::read_to_string(root.join("make/target-list.json")).expect("read make target list");
    let target_list_json: serde_json::Value =
        serde_json::from_str(&target_list_text).expect("parse make target list");
    let snapshot = target_list_json["public_targets"]
        .as_array()
        .expect("target list public_targets")
        .iter()
        .filter_map(|v| v.as_str())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    assert_eq!(
        observed, snapshot,
        "make/target-list.json must snapshot the curated `bijux-dev-atlas make list` surface"
    );
    let mut sorted = observed.clone();
    sorted.sort();
    assert_eq!(
        observed, sorted,
        "make wrapper target list must use deterministic sorted ordering"
    );
}

#[test]
fn every_curated_wrapper_must_be_documented_and_registered() {
    let root = repo_root();
    let list_output = Command::new(env!("CARGO_BIN_EXE_bijux-dev-atlas"))
        .args(["make", "list", "--format", "json"])
        .output()
        .expect("run make list");
    assert!(list_output.status.success());
    let list_payload: serde_json::Value =
        serde_json::from_slice(&list_output.stdout).expect("parse make list payload");
    let curated = list_payload["public_targets"]
        .as_array()
        .expect("public_targets array")
        .iter()
        .filter_map(|v| v.as_str())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let docs_registry_text = fs::read_to_string(root.join("configs/make/public-targets.json"))
        .expect("read configs/make/public-targets.json");
    let docs_registry: serde_json::Value =
        serde_json::from_str(&docs_registry_text).expect("parse public-targets");
    let documented = docs_registry["public_targets"]
        .as_array()
        .expect("public_targets array")
        .iter()
        .filter_map(|row| row.get("name").and_then(|v| v.as_str()))
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();

    let make_registry_text = fs::read_to_string(root.join("configs/ops/make-target-registry.json"))
        .expect("read configs/ops/make-target-registry.json");
    let make_registry: serde_json::Value =
        serde_json::from_str(&make_registry_text).expect("parse make target registry");
    let registered = make_registry["targets"]
        .as_array()
        .expect("targets array")
        .iter()
        .filter_map(|row| row.get("name").and_then(|v| v.as_str()))
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();

    let mut missing_docs = Vec::new();
    let mut missing_registry = Vec::new();
    let mut invalid_names = Vec::new();
    for target in curated {
        if !documented.contains(&target) {
            missing_docs.push(target.clone());
        }
        if !registered.contains(&target) {
            missing_registry.push(target.clone());
        }
        let valid_name = target
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
        if !valid_name {
            invalid_names.push(target);
        }
    }
    assert!(
        missing_docs.is_empty(),
        "every curated wrapper must be documented in configs/make/public-targets.json: {}",
        missing_docs.join(", ")
    );
    assert!(
        missing_registry.is_empty(),
        "every curated wrapper must be registered in configs/ops/make-target-registry.json: {}",
        missing_registry.join(", ")
    );
    assert!(
        invalid_names.is_empty(),
        "wrapper names must match lowercase kebab-case naming: {}",
        invalid_names.join(", ")
    );
}
