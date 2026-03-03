# SSOT Boundaries

- Owner: `bijux-atlas-platform`
- Audience: `contributors`
- Status: `active`

## Canonical Boundaries

- `docs/`: user-facing and contributor-facing product documentation.
- `ops/`: operational specifications, inventories, runbooks, and evidence contracts.
- `configs/`: machine-consumed configuration contracts and policy inputs.
- repository root: minimal entrypoint and legal/compliance documents only.

## Governance Index Authority

The canonical governance index is `docs/governance/index.md`.
Any governance landing page under `ops/` must be a redirect stub to that page.

## Duplication Rule

Narrative content lives in one canonical location. Other locations must use short stubs that link to canonical content.

## Root Surface Rule

Root markdown files are controlled by `configs/layout/root-markdown-allowlist.json`.
No root markdown additions are allowed without updating that allowlist and contract references in the same change.
