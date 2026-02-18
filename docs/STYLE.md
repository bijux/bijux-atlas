# Documentation Style

- Owner: `docs-governance`

## What

Authoring standard for naming, tone, depth, and placement of documentation.

## Taxonomy

- `docs/product/`: product behavior, promises, user-facing semantics.
- `docs/architecture/`: system structure, boundaries, design decisions.
- `docs/contracts/`: SSOT contracts and generated contract surfaces.
- `docs/operations/`: deploy/run/observe/recover procedures.
- `docs/development/`: contributor workflow, tooling, repo governance.
- `docs/science/`: domain/scientific semantics and biological reference intent.

## Naming

- Use kebab-case filenames.
- Use `INDEX.md` for section entrypoints.
- Prefer one canonical page per workflow; other pages should link to that canonical page.

## Tone

- Write in direct imperative style.
- Prefer deterministic commands (`make <target>`).
- Avoid placeholders in published docs.

## Depth Contract

Every normative doc should answer:
- `What`
- `Why`
- `How to verify`
- `Failure modes`

## Placement Rules

- Put drafts/placeholders under `docs/_drafts/` only.
- Put generated artifacts under `docs/_generated/` only.
- Link section-to-section through `INDEX.md` pages where possible.
