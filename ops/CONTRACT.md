# Ops Reference Contract

- Owner: `bijux-atlas-operations`

## What

Defines what "ops reference-grade" means for local and CI operations.

## Reference-grade requirements

- Idempotent: rerunning `make ops-full` succeeds without manual cleanup.
- Reproducible: pinned tool versions and deterministic run-id/namespace.
- Artifacts-first: every run writes metadata + evidence under `artifacts/ops/<run-id>/`.
- Gated: ops checks are make-target driven and CI enforceable.
- Non-interactive in CI: no prompts during CI ops runs.

## Run identity

- Run ID format: `atlas-ops-YYYYMMDD-HHMMSS`.
- Namespace format: one namespace per run (`$OPS_NAMESPACE`, default = run ID).
- Safety namespace pattern: `atlas-ops-*`.

kind-cluster-contract-hash: `b7cbaefe788fae38340ef3aa0bc1b79071b8da6f14e8379af029ac1a3e412960`

## Modes

- `OPS_MODE=fast`: short/PR-safe path.
- `OPS_MODE=full`: nightly-grade path with longer checks.
- `OPS_DRY_RUN=1`: print actions instead of mutating state.

## Failure behavior

- Failures must produce bundles (events, pod state, logs, helm manifests).
- Metadata must include git sha, image digest, policy hash, dataset hash, and tool versions.

## Verification

```bash
make ops-tools-check
make ops-tools-print
make ops-ref-grade-local
make ops-report
```

### Required Ref-Grade Local Sequence

`make ops-ref-grade-local` MUST execute these gates in order:

1. `make ops-tools-check`
2. `make ops-kind-validate`
3. `make ops-stack-validate`
4. `make ops-deploy PROFILE=local`
5. `make ops-publish DATASET=medium`
6. `make ops-warm`
7. `make ops-smoke`
8. `make ops-k8s-tests` (PR subset via `make ops-ref-grade-pr`, full via `make ops-ref-grade-nightly`)
9. `make ops-load-smoke`
10. `make ops-observability-validate`
