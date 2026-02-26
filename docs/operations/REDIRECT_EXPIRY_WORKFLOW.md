# Redirect Expiry Workflow

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Purpose: `govern redirect stubs and temporary doc redirects with explicit expiry and replacement paths`
- Consumers: `checks_ops_human_workflow_maturity`

## Workflow

1. Add redirect only when a canonical destination exists.
2. Record expiry date and replacement path in the redirect document.
3. Link the redirect from the relevant index only while migration is active.
4. Remove redirect before or on expiry date and update all inbound links.

## Required Metadata

- `Expiry Date`
- `Replacement Path`
- `Owner`

## Enforcement Links

- `checks_ops_human_workflow_maturity`
- `checks_docs_markdown_link_targets_exist`
