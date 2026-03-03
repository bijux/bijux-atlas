# Release Trust Root

- Owner: `bijux-atlas-operations`
- Type: `concept`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Last changed: `2026-03-03`
- Reason to exist: define what a consumer is expected to trust when verifying a release bundle.

Related ops contracts: `OPS-ROOT-023`, `REL-PROV-001`.

## Trust Root

- `configs/rust/toolchain.json` is the canonical toolchain inventory for governed release generation.
- `release/signing/policy.yaml` is the canonical statement of which artifacts must be covered.
- `release/signing/checksums.json` is the integrity ledger consumers compare against the published files.
- `release/provenance.json` binds the release bundle to one git SHA and one governance version.

## Deterministic Metadata

- Release tarballs are built with stable member ordering.
- Member timestamps are normalized to zero.
- Ownership fields are normalized to empty names and zero numeric IDs.
- Verification commands are local-only and do not depend on remote trust services.
