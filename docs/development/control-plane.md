# Control Plane

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: define what the control plane owns, forbids, and guarantees.

## Why It Exists

The control plane centralizes operational and governance execution so repository behavior stays deterministic, inspectable, and auditable.

## Control-Plane Contract

### Responsibilities

- Provide stable command orchestration for checks, docs, configs, policies, and ops lanes.
- Generate deterministic machine-readable evidence artifacts.
- Enforce policy and contract checks before merge and release lanes.
- Gate effectful operations behind explicit commands and manifests.
- Keep CI lane behavior explicit and reproducible.

### Non-Responsibilities

- Running hidden script entrypoints as a production surface.
- Mutating runtime data semantics through control-plane shortcuts.
- Bypassing required contracts in merge or release lanes.
- Owning runtime business logic in API/query/store crates.
- Allowing undocumented side effects in validation workflows.

## Surfaces

- Commands: `bijux dev atlas ...`
- Gates: check, docs, configs, policy, and ops validation lanes
- Reports: JSON evidence outputs and human summaries
- Artifacts: lane-scoped generated outputs for CI and local triage

## Command Families

- `check`
- `docs`
- `configs`
- `ops`
- `policies`

## Lanes

- Local lane: fast contributor feedback.
- Pull request lane: required contract and policy gates.
- Merge lane: full required validation with stable outputs.
- Release lane: highest assurance and operator-ready evidence.

### Required Contracts Lane Map

- `local`: fast local validation before broader lanes.
- `pr`: required contracts and policy gates for pull requests.
- `merge`: required contracts plus broader integration confidence.
- `release`: required contracts and release-readiness evidence.

## Repro Command Pattern

Every failing gate must have a reproducible command line in output or linked docs.

```bash
bijux dev atlas check run --group repo --json-report artifacts/evidence/checks/repo.json
```

## Triage

Use [Debugging Locally](debugging-locally.md) for reproduce -> inspect -> fix flow.

## Effect Boundaries

- `ops/` stores operational data, schemas, and runbooks.
- Effectful ops actions run through explicit `ops` surfaces.
- User-facing operations use `bijux dev atlas ops ...` (or thin `make` wrappers), not raw scripts.

## Security Model

- No implicit network dependency for core contributor checks.
- Least-privilege effect surfaces for Docker/Helm/Kind workflows.
- Explicit CI mode behavior for deterministic machine-readable outputs.

## Performance Budget

- Local contributor lanes should remain practical for iterative use.
- Merge/release lanes can be stricter but must keep deterministic outputs.
- Heavy suites must remain explicitly labeled and scoped.

## Extensibility Model

- New suites must declare contract ownership and output format.
- New checks must include deterministic ordering and evidence schema.
- New lanes must map to clear contributor/operator intent.

## Known Limitations

See [Known Limitations](known-limitations.md) for currently intentional gaps.

## What This Page Is Not

This page is not a command reference and not an incident runbook.

## Next steps

- Runtime boundaries: [Architecture Boundaries](../architecture/boundaries.md)
- CI behavior: [CI Overview](ci-overview.md)
- Terms: [Glossary](../glossary.md)

## What to Read Next

- Runtime boundaries: [Architecture Boundaries](../architecture/boundaries.md)
- CI behavior: [CI Overview](ci-overview.md)
- Core terms: [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
