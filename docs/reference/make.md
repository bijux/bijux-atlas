# Make Reference

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: define the stable `make` entrypoint model.

## Model

`make` targets are wrappers over canonical control-plane commands.

## Target groups

- Ops targets: deploy, readiness, observability, load, release.
- Docs targets: docs checks and inventory generation.
- Contracts targets: schema and contract validation.

## Rules

- Prefer published targets only.
- Do not depend on ad-hoc local aliases in docs.
- Use command references for underlying CLI details.

## Next

- [Commands Reference](commands.md)
- [Operations Surface Reference](ops-surface.md)
