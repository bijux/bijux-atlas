# Operations Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## What

Canonical operations handbook for this repository. Human-facing operations guidance lives under `docs/operations/**`.

## Why

Provides one stable operations surface linking deployment, observability, load, runbooks, and security.

## Scope

Kubernetes operations, observability posture, load validation, incident runbooks, and security practices.

## Non-goals

Does not define product semantics or internal crate APIs.

## Contract Boundary

Operational policies are enforced by contracts; docs are walkthroughs.
Policy statements in this directory must link to executable contract IDs such as `OPS-ROOT-017`, `OPS-ROOT-022`, and `OPS-ROOT-023`.

## Contracts

- [Kubernetes](k8s/INDEX.md)
- [Kubernetes Operations](kubernetes.md)
- [Observability](observability/slo.md)
- [SLO](slo/INDEX.md)
- [Load](load/k6.md)
- [Runbooks](runbooks/INDEX.md)
- [Security](security/security-posture.md)
- [Ops Filesystem Layout](ops-layout.md)
- [Ops Entrypoints](entrypoints.md)
- [Command Surface Reference](reference/commands.md)
- [Ops Surface Reference](reference/ops-surface.md)
- [Ops Map](ops-map.md)
- [Tools Reference](reference/tools.md)
- [Toolchain Reference](reference/toolchain.md)
- [Pins Reference](reference/pins.md)
- [Gates Reference](reference/gates.md)
- [Drills Reference](reference/drills.md)
- [Schema Index Reference](reference/schema-index.md)
- [Evidence Model Reference](reference/evidence-model.md)
- [What Breaks If Removed Reference](reference/what-breaks-if-removed.md)
- [How To Run Locally](how-to-run-locally.md)
- [Full Stack Local](full-stack-local.md)
- [Local Stack (Make Only)](local-stack.md)
- [Health Endpoint Semantics](health-endpoint-semantics.md)
- [Canonical Workflows](canonical-workflows.md)
- [Runtime Config](config.md)
- [Release Workflows](release-workflows.md)
- [K8s Tests](e2e/k8s-tests.md)
- [Observability](observability/INDEX.md)
- [Production Readiness Checklist](production-readiness-checklist.md)
- [Ops Acceptance Checklist](ops-acceptance-checklist.md)
- [Retention and GC](retention-and-gc.md)
- [Evidence Policy](evidence-policy.md)
- [Input Sources](input-sources.md)
- [Cache Topology](cache-topology.md)
- [Policy Violation Triage](policy-violation-triage.md)
- [No Direct Path Usage](no-direct-path-usage.md)
- [Platform Handover](platform-handover.md)
- [Ops System](ops-system/INDEX.md)
- [Docs Convergence Policy](DOCS_CONVERGENCE_POLICY.md)
- [Docs Structure Freeze](DOCS_STRUCTURE_FREEZE.md)

## Topic Registry

| Topic | Canonical Doc |
|---|---|
| k8s chart/install/gates | `docs/operations/k8s/INDEX.md` |
| observability contracts/drills | `docs/operations/observability/INDEX.md` |
| SLI/SLO policy | `docs/operations/slo/INDEX.md` |
| load suites/baselines | `docs/operations/load/INDEX.md` |
| e2e composition | `docs/operations/e2e/INDEX.md` |
| runbooks | `docs/operations/runbooks/INDEX.md` |
| security operations | `docs/operations/security/INDEX.md` |

## Failure modes

Missing operational references causes inconsistent incident response and unsafe deployments.

## How to verify

```bash
$ bijux dev atlas docs doctor --format json
```

Expected output: docs doctor reports status `ok` and no operations docs drift.

## See also

- [Product SLO Targets](../product/slo-targets.md)
- [Contracts Metrics](../contracts/metrics.md)
- [Terms Glossary](../_style/terms-glossary.md)
- `ops-ci`
