// SPDX-License-Identifier: Apache-2.0

use super::*;

fn check(source: &str) -> Result<Vec<Violation>, CheckError> {
    let pins = BTreeMap::from([("actions/checkout".to_string(), "a".repeat(40))]);
    workflow_action_violations(source, &pins, Path::new(".github/workflows/contract.yml"))
}

#[test]
fn quoted_and_unquoted_action_keys_enforce_the_same_inventory() {
    let source = format!(
        "env:\n  uses: unrelated-environment-value\njobs:\n  gate:\n    steps:\n      - uses: actions/checkout@{0}\n      - \"uses\": \"actions/checkout@{0}\"\n      - uses: ./local-action\n      - uses: docker://alpine:3\n      - run: |\n          uses: unrelated-script-text\n", "a".repeat(40)
    );
    assert!(check(&source).unwrap().is_empty());
}

#[test]
fn quoted_action_missing_from_inventory_is_rejected() {
    let source = format!(
        "jobs:\n  gate:\n    steps:\n      - \"uses\": ruby/setup-ruby@{}\n",
        "a".repeat(40)
    );
    let violations = check(&source).unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(
        violations[0].code.as_str(),
        "workflow_action_not_allowlisted"
    );
}

#[test]
fn quoted_action_with_wrong_or_floating_pin_is_rejected() {
    for (pin, code) in [
        ("b".repeat(40), "workflow_action_sha_mismatch"),
        ("v7".to_string(), "workflow_action_not_sha_pinned"),
    ] {
        for source in [
            format!("jobs:\n  gate:\n    steps:\n      - \"uses\": actions/checkout@{pin}\n"),
            format!("jobs:\n  gate:\n    \"uses\": actions/checkout@{pin}\n"),
        ] {
            let violations = check(&source).unwrap();
            assert_eq!(violations.len(), 1);
            assert_eq!(violations[0].code.as_str(), code);
        }
    }
}

#[test]
fn malformed_workflow_cannot_silently_skip_inventory_validation() {
    assert!(check("jobs: [broken").is_err());
}
