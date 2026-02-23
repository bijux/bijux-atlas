# Ops Execution Model (Target)

## Goal

`atlasctl` is the single execution control-plane for repository ops actions.

Target state:
- no shell wrapper entrypoints under `ops/`
- `ops/` contains data, manifests, fixtures, contracts, schemas, and test inputs only
- `packages/atlasctl/src/atlasctl/commands/ops/**` owns orchestration behavior

## Execution Flow

1. CLI parser resolves `atlasctl ops <area> <action>`.
2. Command layer builds a typed ops runtime request.
3. Ops runtime executes via `core.process` / `core.exec` / `core.fs` helpers.
4. Reports are emitted under allowed artifact/evidence roots.
5. Report payloads are validated against atlasctl-owned schemas.

## Boundary Rules

- `commands/ops/**` is orchestration code, not a shell-script registry.
- `commands/ops/**` may import only:
  - `atlasctl.core.*`
  - `atlasctl.contracts.*`
  - `atlasctl.reporting.*`
  - `atlasctl.registry.*`
  - `atlasctl.commands._shared`
  - `atlasctl.commands.ops.*` (intra-domain)
- `commands/ops/**` must not import `atlasctl.cli.*`.
- Temporary migration glue lives only under `commands/ops/internal/`.

## Shell Policy (Residual)

Shell may still appear for:
- test fixtures/assets
- hermetic shell snippets embedded in atlasctl-owned modules (transitional)

These are implementation details. Public entrypoints remain stable atlasctl CLI subcommands.

## Reporting

Ops lanes emit structured reports with:
- run metadata (`run_id`, timestamps, status)
- stable step/check rows
- deterministic ordering
- schema validation (`atlasctl.ops-report.v1`)

### Summary Report Ownership

- `atlasctl.reporting.*` is the canonical home for summary/unified report generation.
- `commands/ops/**` emits area/lane detail payloads and evidence, not global summary aggregation.

### Evidence Layout

Canonical ops evidence layout:

- `artifacts/evidence/<area>/<run_id>/...`

Use atlasctl-owned helpers to materialize these paths.

### Policy

- Ops must not write outside approved evidence/generated roots.
- Ops must not call `subprocess` directly unless temporarily allowlisted; prefer `atlasctl.core.process`.
- Ops external tool invocations must be declared in `configs/ops/external-tools-allowlist.json`.
