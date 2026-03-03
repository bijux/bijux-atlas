# Safe Logging Guidelines

- Owner: `bijux-atlas-security`
- Type: `policy`
- Audience: `contributors`
- Stability: `stable`
- Last verified against: `main@f5f3bd4471d8bf4dbf13fef4381ef6bfda2480a2`
- Reason to exist: define the only safe structured fields for audit and operational logs.

## Rules

- New audit or structured log fields must be added to `configs/observability/log-safe-fields.yaml` before
  code emits them.
- If a field is not safe, classify it in `configs/security/data-classification.yaml` and do not emit it in
  audit records.
- Never log raw credentials, bearer tokens, API keys, signatures, direct client IPs, or email addresses.
- Prefer stable identifiers, booleans, and bounded enums over raw payload fragments.

## Verify

- Run `cargo run -q -p bijux-dev-atlas -- security validate --format json`.
- Confirm `OBS-LOG-INV-001=true` and `SEC-RED-002=true`.

## Rollback

- Remove the unreviewed field from the logging call site.
- Regenerate the security reports so the inventory returns to a governed state.
