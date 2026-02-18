# Operations Index

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## What

Canonical entrypoint for operating atlas in production.

## Why

Provides one stable operations surface linking deployment, observability, load, runbooks, and security.

## Scope

Kubernetes operations, observability posture, load validation, incident runbooks, and security practices.

## Non-goals

Does not define product semantics or internal crate APIs.

## Contracts

- [Kubernetes](k8s/INDEX.md)
- [Kubernetes Operations](kubernetes.md)
- [Observability](observability/slo.md)
- [Load](load/k6.md)
- [Runbooks](runbooks/INDEX.md)
- [Security](security/security-posture.md)
- [Ops Filesystem Layout](ops-layout.md)
- [Full Stack Local](full-stack-local.md)
- [Local Stack (Make Only)](local-stack.md)
- [Health Endpoint Semantics](health-endpoint-semantics.md)
- [Canonical Workflows](canonical-workflows.md)
- [Release Workflows](release-workflows.md)
- [k6 Workflows](load/k6.md)
- [K8s Tests](e2e/k8s-tests.md)
- [Observability](observability/INDEX.md)
- [Production Readiness Checklist](production-readiness-checklist.md)
- [Ops Acceptance Checklist](ops-acceptance-checklist.md)
- [Retention and GC](retention-and-gc.md)
- [Input Sources](input-sources.md)
- [Cache Topology](cache-topology.md)
- [Policy Violation Triage](policy-violation-triage.md)
- [No Direct Path Usage](no-direct-path-usage.md)
- [Migration Note](migration-note.md)
- [Platform Handover](platform-handover.md)

## Failure modes

Missing operational references causes inconsistent incident response and unsafe deployments.

## How to verify

```bash
$ make docs
```

Expected output: operations links resolve and docs checks pass.

## See also

- [Product SLO Targets](../product/slo-targets.md)
- [Contracts Metrics](../contracts/metrics.md)
- [Terms Glossary](../_style/terms-glossary.md)
- `ops-ci`
