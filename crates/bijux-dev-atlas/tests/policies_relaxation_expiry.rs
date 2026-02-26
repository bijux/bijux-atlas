// SPDX-License-Identifier: Apache-2.0

use bijux_dev_atlas::policies::{validate_relaxation_expiry, PolicyValidationError};

#[test]
fn relaxation_expiry_is_enforced() {
    let expired = serde_json::json!({
      "relaxations": [
        {
          "policy_id":"repo.max_loc_hard",
          "reason":"temporary exception",
          "expires_on":"2020-01-01"
        }
      ]
    });
    let err = validate_relaxation_expiry(&expired, "2026-02-24").expect_err("must fail");
    assert!(err.to_string().contains("expired on `2020-01-01`"));

    let valid = serde_json::json!({
      "relaxations": [
        {
          "policy_id":"repo.max_loc_hard",
          "reason":"temporary exception",
          "expires_on":"2027-01-01"
        }
      ]
    });
    assert!(validate_relaxation_expiry(&valid, "2026-02-24").is_ok());
}

#[test]
fn relaxation_expiry_rejects_invalid_date_format() {
    let invalid = serde_json::json!({
      "relaxations": [
        {
          "policy_id":"repo.max_loc_hard",
          "reason":"temporary exception",
          "expires_on":"2026/02/24"
        }
      ]
    });
    let err = validate_relaxation_expiry(&invalid, "2026-02-24").expect_err("must fail");
    assert!(matches!(err, PolicyValidationError(_)));
}
