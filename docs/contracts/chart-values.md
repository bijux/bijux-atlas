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
- `cache`
- `catalog`
- `catalogPublishJob`
- `concurrency`
- `datasetWarmupJob`
- `extraEnv`
- `hpa`
- `image`
- `networkPolicy`
- `nodeLocalSsdProfile`
- `nodeSelector`
- `pdb`
- `podSecurityContext`
- `priorityClassName`
- `rateLimits`
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
offline:
  enabled: true
server:
  cachedOnlyMode: true
```

Expected output: values keys validate against `CHART_VALUES.json`.

## How to verify

```bash
$ make ssot-check
$ make docs-freeze
```

Expected output: both commands exit status 0 and print contract generation/check success.

## See also

- [Contracts Index](_index.md)
- [SSOT Workflow](SSOT_WORKFLOW.md)
- [Terms Glossary](../_style/TERMS_GLOSSARY.md)
