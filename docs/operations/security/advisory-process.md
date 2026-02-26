# Security Advisory Publication Workflow

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Concept IDs: concept.security-coordination

Use `.github/workflows/security-advisory-publication.yml` to publish advisory records.

## Flow
1. Fill advisory details from triage.
2. Trigger workflow dispatch with advisory metadata.
3. Workflow renders `docs/security/advisories/<ID>.md` and opens a PR.
4. Review + merge, then announce per response policy.

This keeps advisory publication auditable and deterministic.

## See also

- `ops-ci`
