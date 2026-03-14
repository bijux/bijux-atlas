// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

#[test]
fn threat_model_registry_files_exist_and_are_well_formed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    let assets = root.join("ops/security/threat-model/assets.yaml");
    let threats = root.join("ops/security/threat-model/threats.yaml");
    let mitigations = root.join("ops/security/threat-model/mitigations.yaml");
    let taxonomy = root.join("ops/security/threat-model/classification-taxonomy.yaml");
    let registry = root.join("ops/security/threat-model/threat-registry.yaml");

    for path in [&assets, &threats, &mitigations, &taxonomy, &registry] {
        assert!(
            path.exists(),
            "missing required threat model file: {}",
            path.display()
        );
        let raw = std::fs::read_to_string(path).expect("read threat model file");
        let value: serde_yaml::Value = serde_yaml::from_str(&raw).expect("parse yaml");
        assert!(
            value.get("schema_version").is_some(),
            "missing schema_version in {}",
            path.display()
        );
    }
}

#[test]
fn threat_registry_ids_cover_threat_entries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    let threats: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(root.join("ops/security/threat-model/threats.yaml"))
            .expect("read threats"),
    )
    .expect("parse threats");
    let registry: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(root.join("ops/security/threat-model/threat-registry.yaml"))
            .expect("read registry"),
    )
    .expect("parse registry");

    let threat_ids = threats
        .get("threats")
        .and_then(serde_yaml::Value::as_sequence)
        .expect("threat sequence")
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_yaml::Value::as_str))
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();

    let registry_ids = registry
        .get("threat_ids")
        .and_then(serde_yaml::Value::as_sequence)
        .expect("registry sequence")
        .iter()
        .filter_map(serde_yaml::Value::as_str)
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(threat_ids, registry_ids, "threat registry mismatch");
}
