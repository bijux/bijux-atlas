# Docs Charter

- Owner: `bijux-atlas-docs`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: define the documentation product contract and non-negotiable quality rules.

## Docs Product Contract

Atlas documentation is a reader product for humans.
Tooling artifacts are not part of the reader documentation surface.

## Audience Model

Only these audiences are valid in stable docs:

- `user`
- `operator`
- `contributor`

## Page Type Model

Only these page types are valid:

- `concept`
- `guide`
- `runbook`
- `reference`
- `policy`
- `adr`

## Stability Model

Stable docs use exactly one stability level:

- `draft`
- `stable`
- `deprecated`

## Ownership Rule

Every `stable` page must have an explicit owner.

## Reader Spine

The reader spine is fixed and pinned from `docs/index.md`:

- `docs/start-here.md`
- `docs/product/index.md`
- `docs/architecture/index.md`
- `docs/api/index.md`
- `docs/operations/index.md`
- `docs/development/index.md`
- `docs/reference/index.md`
- `docs/glossary.md`

## Navigation and Reachability Rules

- Core workflows must be reachable in at most 3 clicks.
- Every non-draft page must be reachable from a section index.
- `_generated/**` is never part of reader navigation.
- Governance content is contributor-only and excluded from reader paths.
- Each section index must list no more than 10 curated links.

## Workflow Rules

- Each audience has one canonical golden path doc.
- Every procedural page must include a `Verify success` section.
- Every operations procedure must include rollback steps or an explicit `No rollback` statement.
- Core pages must include a `Last verified against` marker.
