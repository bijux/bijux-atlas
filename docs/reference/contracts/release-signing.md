# Release Signing Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Last changed: `2026-03-03`
- Reason to exist: document the governed release signing and provenance contract set.

## Contract Set

- `REL-SIGN-001`: the checksum list exists and covers every required signed artifact.
- `REL-SIGN-002`: checksum entries match the current contents of the referenced files.
- `REL-SIGN-003`: the sign report validates and records every required signing target.
- `REL-SIGN-004`: a freshly generated bundle passes release verification end-to-end.
- `REL-PROV-001`: `release/provenance.json` exists and matches the governed provenance schema.

## Validation Surface

- `release sign --evidence release/evidence` generates the checksum ledger, provenance statement, and sign report.
- `release verify --evidence release/evidence/bundle.tar` validates the signing surface and delegates to `ops evidence verify`.
- `release/signing/policy.yaml` is the source of truth for what must be covered.

## Governed Files

- `release/signing/policy.yaml`
- `release/signing/checksums.json`
- `release/signing/release-sign.json`
- `release/signing/release-verify.json`
- `release/provenance.json`
- `configs/contracts/release/*.schema.json`
