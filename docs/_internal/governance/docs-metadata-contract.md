# Docs Metadata Contract

- Owner: `docs-governance`
- Status: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`

## Required Metadata

Every governed docs page must provide these metadata keys:

- `owner`
- `status` (`draft` or `stable`)
- `last_verified` (`YYYY-MM-DD`)
- `scope`

Schema source of truth: `configs/contracts/docs-metadata.schema.json`.

## Orphan Allowlist

Intentional orphan pages must be declared in `docs/_internal/policies/orphan-allowlist.json` with:

- `path`
- `owner`
- `justification`
- `expires_on` (`YYYY-MM-DD`)

Expired allowlist entries are surfaced by `bijux-dev-atlas docs health-dashboard --allow-write`.
