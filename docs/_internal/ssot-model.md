# SSOT Model

- Owner: `bijux-atlas-platform`
- Audience: `contributors`
- Stability: `stable`

## Canonical Boundaries

- `docs/` is user-facing institutional documentation and contributor guidance.
- `ops/` is operational specifications, inventories, runbooks, policies, and evidence references only.
- `configs/` is machine-consumed policy and configuration source.
- repository root markdown is restricted to a governed allowlist.

## Root Markdown Allowlist

Root markdown files are governed by `configs/layout/root_markdown_allowlist.json`.
No new root markdown file is allowed without updating the allowlist in the same change.

## Ops Documentation Guardrails

- Narrative documentation is forbidden under `ops/`.
- Onboarding documentation is forbidden under `ops/`.
- Every ops markdown document must declare `Doc-Class` front matter.
- Allowed classes: `spec`, `runbook`, `policy`, `evidence`, `stub`.

## Linking Rules

- `docs/**` may link to stable operational specs under `ops/**`.
- `ops/**` may link to stable public pages under `docs/**` only when needed.
- `ops/**` must not use `docs/_internal/**` as canonical user guidance.
- `docs/**` must not treat `_generated` or `_generated.example` ops paths as SSOT.
