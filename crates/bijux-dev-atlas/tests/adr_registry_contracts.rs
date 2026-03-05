// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace")
        .parent()
        .expect("repo")
        .to_path_buf()
}

#[test]
fn adr_registry_includes_required_doctrine_decisions() {
    let root = repo_root();
    let path = root.join("docs/governance/adr-registry.md");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    for decision in [
        "ADR-0002: Automation engine boundary",
        "ADR-0003: User and dev CLI separation",
        "ADR-0004: Generated reference model",
        "ADR-0005: Ops artifact directory purity",
        "ADR-0006: Tutorial structure and consolidation",
    ] {
        assert!(
            text.contains(decision),
            "adr registry must include required decision `{decision}`"
        );
    }
}
