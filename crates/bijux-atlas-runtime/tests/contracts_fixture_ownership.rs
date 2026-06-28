// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn ingest_and_cli_fixtures_live_with_their_owning_surfaces() {
    let root = workspace_root();

    assert!(
        !root
            .join("crates/bijux-atlas-runtime/tests/fixtures")
            .exists(),
        "runtime crate must not act as a shared fixture warehouse"
    );
    assert!(
        root.join("crates/bijux-atlas-ingest/tests/fixtures/policies")
            .is_dir(),
        "ingest policy fixtures must live under the ingest crate"
    );
    assert!(
        root.join("crates/bijux-atlas-cli/tests/fixtures/qc_edgecases")
            .is_dir(),
        "cli operation QC fixtures must live beside the owning CLI operation tests"
    );
}
