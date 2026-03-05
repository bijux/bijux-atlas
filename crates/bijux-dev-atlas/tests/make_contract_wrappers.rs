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
