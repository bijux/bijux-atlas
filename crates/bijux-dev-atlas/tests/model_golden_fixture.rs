// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::model::RunReport;
use std::fs;
use std::path::PathBuf;

#[test]
fn canonical_report_fixture_parses_and_matches_shape() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/model_fixtures/run_report_canonical.json");
    let text = fs::read_to_string(fixture).expect("fixture");
    let parsed: RunReport = serde_json::from_str(&text).expect("parse");
    assert_eq!(
        parsed.schema_version,
        bijux_dev_atlas::model::schema_version()
    );
    assert_eq!(parsed.summary.total, parsed.results.len() as u64);
    assert_eq!(parsed.run_id.as_str(), "registry_run");
}
