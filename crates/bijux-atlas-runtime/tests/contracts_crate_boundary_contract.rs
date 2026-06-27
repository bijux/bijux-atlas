// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn crate_boundary_contract_document_exists_with_required_sections() {
    let root = repo_root();
    let path = root.join("docs/bijux-atlas/foundations/crate-boundary-contract.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    for required in [
        "# Crate Boundary Contract",
        "## Crate Map",
        "## Ownership Rules",
        "## Dependency Direction",
        "## Enforcement",
        "bijux-atlas",
        "bijux-atlas-cli",
        "bijux-atlas-dev",
        "bijux-atlas-core",
        "bijux-atlas-ops",
        "bijux-atlas-ingest",
        "bijux-atlas-model",
        "bijux-atlas-query",
        "bijux-atlas-api",
        "bijux-atlas-runtime",
        "bijux-atlas-server",
        "bijux-atlas-store",
    ] {
        assert!(
            text.contains(required),
            "crate-boundary contract missing required marker `{required}`"
        );
    }
}

#[test]
fn runtime_identity_constants_match_owned_crates() {
    assert_eq!(bijux_atlas_runtime::CRATE_NAME, "bijux-atlas-runtime");
    assert_eq!(
        bijux_atlas_runtime::adapters::outbound::store::CRATE_NAME,
        "bijux-atlas-store"
    );
    assert_eq!(
        bijux_atlas_runtime::no_randomness_policy(),
        "Randomness is forbidden in bijux-atlas-runtime"
    );
}
