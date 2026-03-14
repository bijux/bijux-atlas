# Security Testing And Monitoring

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Security test lanes

- `cargo test -p bijux-atlas --test server security_input_resilience`
- `cargo test -p bijux-atlas-query parser_fuzz`
- `cargo test -p bijux-atlas-ingest fuzzish_attribute_order_spacing_is_stable`
- `k6 run ops/load/k6/suites/security-penetration-simulation.js`
- `k6 run ops/load/k6/suites/security-malicious-input-suite.js`
- `k6 run ops/load/k6/suites/security-injection-suite.js`
- `k6 run ops/load/k6/suites/security-dos-resilience-suite.js`

## Vulnerability and supply-chain checks

- `cargo audit` for Rust advisories.
- `bijux-dev-atlas security validate` for vulnerability budget, lock posture, pinned actions, SBOM coverage, and scan artifact integrity.

## Runtime monitoring

Track and alert on these event and metric families:

- `authentication_failure_alert`
- `authorization_denied`
- `integrity_violation_alert`
- `tamper_detection_alert`
- `atlas_authentication_failures_total`
- `atlas_authorization_denials_total`
- `atlas_integrity_violations_total`
- `atlas_tamper_detections_total`
