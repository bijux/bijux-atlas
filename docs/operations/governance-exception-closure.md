# Exception Closure

- Owner: `bijux-atlas-governance`
- Type: `procedure`
- Audience: `contributor`
- Stability: `stable`
- Last reviewed: `2026-03-03`
- Reason to exist: define how an exception is removed without losing evidence of why it existed.

## Closure procedure

- Land the code, config, or documentation change that removes the need for the exception.
- Remove the entry from `configs/governance/exceptions.yaml`.
- Append the frozen record to `configs/governance/exceptions-archive.yaml` with `archived_at` and the digest-backed `content_sha256`.
- Keep the same mitigation issue link so reviewers can correlate closure with the original reason.
- Re-run `cargo run -q -p bijux-dev-atlas -- governance exceptions validate --format json`.

## Closure evidence

- Updated `artifacts/governance/exceptions-summary.json`
- Updated `artifacts/governance/exceptions-churn.json`
- Archived record in `configs/governance/exceptions-archive.yaml`
- Mitigation issue showing the underlying gap is closed
