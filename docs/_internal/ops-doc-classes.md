# Ops Document Classes

- Owner: `bijux-atlas-platform`
- Audience: `contributors`
- Stability: `stable`

## Required Front Matter

Every markdown document under `ops/` must include front matter with `Doc-Class`.

```yaml
---
Doc-Class: spec
Owner: team:atlas-ops
---
```

## Allowed Values

- `spec`: durable operational specification.
- `runbook`: operational execution procedure.
- `policy`: enforceable operational policy.
- `evidence`: evidence reference document.
- `stub`: redirect document that points to canonical content in `docs/`.

## Stub Rules

A stub document must remain minimal and link to one canonical destination.
It must not duplicate narrative body content from `docs/`.

## Boundary Rules

- Narrative and onboarding belong in `docs/`.
- `ops/` remains operational and executable in purpose.
