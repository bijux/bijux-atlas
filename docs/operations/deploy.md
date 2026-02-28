# Deploy

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@50be979f`
- Reason to exist: define canonical deployment flow for staging and production clusters.

## Prerequisites

- Release artifact is published and approved.
- Cluster access and namespace policies are configured.
- Required chart values are prepared.

## Install with Helm Defaults

```bash
make ops-deploy
```

Use defaults for baseline environments where no override policy is required.

## Minimal Production Overrides

Use only required production deltas:

- image pin
- replica count
- resource requests/limits
- persistence settings
- observability endpoints

## Values Mapping

Chart values map to runtime config keys documented in [Reference Configs](../reference/configs.md).

## Verify Success

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

Expected outcome: workloads ready, probes healthy, and observability checks pass.

## Rollback

```bash
make stack-down
```

Rollback switches serving state back to last known good release and stops failed deployment surfaces.

## Next

- [Release Workflow](release-workflow.md)
- [Incident Response](incident-response.md)
- [K8s](k8s/index.md)
