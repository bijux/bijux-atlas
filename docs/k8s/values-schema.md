# Chart Values Schema Contract

Allowed top-level keys in `charts/bijux-atlas/values.yaml`:

- `image`
- `replicaCount`
- `service`
- `resources`
- `podSecurityContext`
- `securityContext`
- `cache`
- `rateLimits`
- `concurrency`
- `server`
- `sequenceRateLimits`
- `catalog`
- `store`
- `networkPolicy`
- `serviceMonitor`
- `hpa`
- `pdb`
- `priorityClassName`
- `terminationGracePeriodSeconds`
- `nodeLocalSsdProfile`
- `rollout`
- `catalogPublishJob`
- `datasetWarmupJob`
- `extraEnv`
- `nodeSelector`
- `tolerations`
- `affinity`

Policy:
- New top-level keys require updating this file and `test_values_contract.sh`.
- Default values must stay conservative and production-safe.
