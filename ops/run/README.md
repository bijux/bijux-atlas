# Run Module

Purpose: provide the only human-facing ops script entrypoints (`ops/run/*.sh`) used by Make targets.

Entry points:
- `make ops-check`
- `make ops-smoke`
- `make ops-stack-up`
- `make ops-stack-down`
- `make ops-k8s-suite`
- `make ops-load-suite`
- `make ops-obs-verify`

Contracts:
- `ops/run/CONTRACT.md`

Artifacts:
- Entrypoints must emit logs/artifacts under `ops/_artifacts/<run_id>/` and/or `ops/_generated/`.

Failure modes:
- Missing run context (`RUN_ID`/artifact dir).
- Invalid profile/suite selection.
- Non-canonical direct use of area-private scripts.
