# Chart Values Contract

- Owner: `docs-governance`

## What

Defines the `Chart Values Contract` registry contract.

## Why

Prevents drift between SSOT JSON, generated code, and operational consumers.

## Scope

Applies to producers and consumers of this registry.

## Non-goals

Does not define implementation internals outside this contract surface.

## Contracts

- `affinity`
- `alertRules`
- `cache`
- `catalog`
- `catalogPublishJob`
- `datasetWarmupJob`
- `extraEnv`
- `hpa`
- `image`
- `ingress`
- `metrics`
- `networkPolicy`
- `nodeLocalSsdProfile`
- `nodeSelector`
- `pdb`
- `podSecurityContext`
- `priorityClassName`
- `replicaCount`
- `resources`
- `rollout`
- `securityContext`
- `sequenceRateLimits`
- `server`
- `service`
- `serviceMonitor`
- `store`
- `terminationGracePeriodSeconds`
- `tolerations`

## Failure modes

Invalid or drifted registry content is rejected by contract checks and CI gates.

## Examples

```yaml
# default profile
server:
  cachedOnlyMode: false
  logJson: true

# offline profile
cache:
  initPrewarm:
    enabled: false
server:
  cachedOnlyMode: true
  readinessRequiresCatalog: false
```

Expected output: values keys validate against `CHART_VALUES.json`.

## How to verify

```bash
$ make contracts
$ make contracts-docs
```

Expected output: both commands exit status 0 and print contract generation/check success.

## See also

- [Contracts Index](index.md)
- [SSOT Workflow](../../_internal/ops/governance/repository/contract-ssot-workflow.md)
- [Terms Glossary](../../glossary.md)
