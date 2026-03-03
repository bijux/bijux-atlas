// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

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
    let makefile = fs::read_to_string(repo_root().join("make/contracts.mk")).expect("read make/contracts.mk");
    assert!(makefile.contains("contract: _contracts_guard"));
    assert!(makefile.contains("contract-effect: _contracts_guard"));
    assert!(makefile.contains("contract-all: _contracts_guard"));
    assert!(makefile.contains("contract-list: _contracts_guard"));
    assert!(makefile.contains("contract-report: _contracts_guard"));
    assert!(makefile.contains("$(DEV_ATLAS) --output-format $(FORMAT) contract run --mode static"));
    assert!(makefile.contains("$(DEV_ATLAS) --output-format $(FORMAT) contract run --mode effect --effects-policy allow"));
    assert!(makefile.contains("$(DEV_ATLAS) --output-format $(FORMAT) contract run --mode all --effects-policy allow"));
    assert!(makefile.contains("$(DEV_ATLAS) --output-format $(FORMAT) contract list"));
    assert!(makefile.contains("$(DEV_ATLAS) --output-format $(FORMAT) contract report --last --artifacts-root $(CONTRACTS_ARTIFACT_ROOT)"));
}

#[test]
fn deprecated_make_contract_targets_warn_and_delegate() {
    let makefile = fs::read_to_string(repo_root().join("make/contracts.mk")).expect("read make/contracts.mk");
    assert!(makefile.contains("contracts: ## Deprecated alias for make contract"));
    assert!(makefile.contains("contracts-effect: ## Deprecated alias for make contract-effect"));
    assert!(makefile.contains("contracts-all: ## Deprecated alias for make contract-all"));
    assert!(makefile.contains("deprecated: use \\`make contract\\`"));
    assert!(makefile.contains("deprecated: use \\`make contract-effect\\`"));
    assert!(makefile.contains("deprecated: use \\`make contract-all\\`"));
}

#[test]
fn canonical_make_contract_targets_do_not_parse_output() {
    let makefile = fs::read_to_string(repo_root().join("make/contracts.mk")).expect("read make/contracts.mk");
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
