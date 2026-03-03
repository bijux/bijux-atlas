# Release Signing

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Last changed: `2026-03-03`
- Reason to exist: define how release integrity artifacts are generated and how consumers verify them offline.

## Prereqs

- A freshly generated evidence bundle under `release/evidence/`
- `cargo`, `python3`, and the `bijux-dev-atlas` binary available locally
- A clean working tree for the release artifact files you intend to publish

## Install

Generate the evidence bundle first:

```bash
cargo run -q -p bijux-dev-atlas -- ops evidence collect --allow-subprocess --allow-write --run-id release_candidate --format json
```

Generate the signing artifacts from the governed signing policy:

```bash
cargo run -q -p bijux-dev-atlas -- release sign --evidence release/evidence --format json
```

This writes:

- `release/signing/checksums.json`
- `release/signing/release-sign.json`
- `release/provenance.json`

The signing mechanism is currently the governed checksum ledger described in `release/signing/policy.yaml`. Consumers verify artifacts by matching the recorded SHA-256 values against the published files.

## Verify

Run offline verification of the signed release surface:

```bash
cargo run -q -p bijux-dev-atlas -- release verify --evidence release/evidence/bundle.tar --format json
```

This checks:

- the checksum list exists and covers all required targets
- the recorded checksums match the current files
- the provenance statement is present and well-formed
- the evidence bundle itself still passes `ops evidence verify`

The verification path is intentionally local-only. It does not require network access.

## Rollback

If signing verification fails:

1. Re-run `ops evidence collect` to rebuild the evidence bundle from the current source.
2. Re-run `release sign`.
3. Re-run `release verify`.
4. If the failure persists, stop promotion and compare the generated files with the last known good release candidate.

## Related Pages

- [Release Evidence](release-evidence.md)
- [Release Signing Contracts](../reference/contracts/release-signing.md)
