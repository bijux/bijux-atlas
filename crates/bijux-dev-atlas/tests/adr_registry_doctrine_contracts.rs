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
fn adr_registry_must_include_doctrine_decisions() {
    let root = repo_root();
    let registry_path = root.join("docs/governance/adr-registry.md");
    let registry = fs::read_to_string(&registry_path).expect("read adr registry");

    let required = [
        "decisions/adr-0002-automation-engine-boundary.md",
        "decisions/adr-0003-user-and-dev-cli-separation.md",
        "decisions/adr-0004-generated-reference-model.md",
        "decisions/adr-0005-ops-artifact-directory-purity.md",
        "decisions/adr-0006-tutorial-structure-and-consolidation.md",
        "decisions/adr-0007-tutorial-automation-domain-migration.md",
        "decisions/adr-0008-client-doc-generation-governance.md",
    ];

    let mut missing = Vec::new();
    for item in required {
        if !registry.contains(item) {
            missing.push(item.to_string());
        }
        let decision_path = root.join("docs/governance").join(item);
        if !decision_path.exists() {
            missing.push(format!("missing file {}", decision_path.display()));
        }
    }

    assert!(
        missing.is_empty(),
        "adr registry doctrine decisions are incomplete:\n{}",
        missing.join("\n")
    );
}
