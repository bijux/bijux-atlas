# Docs Search Tips

- Owner: `docs-governance`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: provide contributor search tips for troubleshooting docs coverage and drift.

## Practical Queries

- Search for orphan-like pages: `rg --files docs | rg '\\.md$'`
- Search missing taxonomy footer: `rg "## Document Taxonomy" docs`
- Search generated-only references: `rg "_generated/" docs`

## Usage Scope

Use this page only for contributor workflows. Reader-facing sections should not link here.
