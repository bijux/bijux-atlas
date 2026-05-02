// SPDX-License-Identifier: Apache-2.0

use bijux_atlas::domain::dataset::{DatasetId, DatasetLifecycleState, DatasetLifecycleTransition};

#[test]
fn dataset_lifecycle_transition_accepts_only_draft_to_published() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let transition = DatasetLifecycleTransition::publish(
        dataset,
        "1714592400".to_string(),
        "atlas-cli".to_string(),
        "manifest-validated-publish".to_string(),
        "a".repeat(64),
        "b".repeat(64),
    );
    transition.validate().expect("valid transition");
}

#[test]
fn dataset_lifecycle_transition_rejects_non_closed_state_machine_edges() {
    let dataset = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset");
    let mut transition = DatasetLifecycleTransition::publish(
        dataset,
        "1714592400".to_string(),
        "atlas-cli".to_string(),
        "manifest-validated-publish".to_string(),
        "a".repeat(64),
        "b".repeat(64),
    );
    transition.from_state = DatasetLifecycleState::Published;
    transition.to_state = DatasetLifecycleState::Published;
    assert!(transition.validate().is_err());
}
