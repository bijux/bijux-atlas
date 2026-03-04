# Security Compliance

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: explain the governed control catalog, evidence matrix, and secret scanning workflow.

## Prereqs

- Control catalog: `security/compliance/controls.yaml`
- Compliance matrix: `security/compliance/matrix.yaml`
- Declared secrets: `configs/security/secrets.json`
- Redaction policy: `configs/security/redaction.json`

## Install

- Validate the threat model: `cargo run -q -p bijux-dev-atlas -- security validate --format json`
- Validate the compliance matrix: `cargo run -q -p bijux-dev-atlas -- security compliance validate --format json`
- Scan a candidate bundle: `cargo run -q -p bijux-dev-atlas -- security scan-artifacts --dir release/evidence --format json`

## Verify

- `security-threat-model.json` reports all `SEC-THREAT-*` and `SEC-RED-*` checks passing.
- `security-compliance.json` reports all `SEC-COMP-*` checks passing.
- `security-artifact-scan.json` reports `SEC-ART-001` passing with zero matches.

## Rollback

- Revert the policy or evidence mapping change.
- Re-run the three security commands until the reports return to `status: ok`.

## Control categories

- Logging and observability controls
- Release evidence and provenance controls
- Network and cluster boundary controls
- Secret handling and redaction controls
