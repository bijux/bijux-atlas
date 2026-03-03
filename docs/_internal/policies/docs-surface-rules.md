# Docs Surface Rules

- Owner: `bijux-atlas-platform`
- Audience: `contributors`

## Navigation Boundary

- MkDocs navigation must contain only `docs/**` paths.
- Direct navigation links to `ops/**` are forbidden.
- Operational sources may be linked only from dedicated docs reference pages.

## Ops Narrative Boundary

- Ops markdown must not contain onboarding or contributor onboarding workflows.
- Onboarding content is canonical in `docs/start-here.md` and `docs/development/**`.

## Inventory Boundary

- Docs pages must not define operational inventory truth.
- Operational inventory SSOT stays in `ops/inventory/**`.
- Docs may reference inventory paths but must not duplicate inventory payloads.
