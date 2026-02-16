# Rollback Playbook

This runbook covers API rollback and catalog rollback.

## API Rollback

1. Freeze rollout progression.
2. Roll back image tag to last known good release.
3. Verify `/healthz`, `/readyz`, and `/metrics` recover.
4. Confirm request error rate and p95 latency return to baseline.

If using Argo Rollouts:

1. Abort current rollout.
2. Promote stable ReplicaSet as active.
3. Keep canary disabled until postmortem completes.

## Catalog Rollback

1. Re-point catalog to prior immutable catalog revision.
2. Invalidate catalog cache by restarting pods or forcing refresh.
3. Verify dataset manifest checksums and schema versions.
4. Run smoke queries for pinned datasets.

## Joint Rollback (API + Catalog)

1. Roll back API first to stabilize query behavior.
2. Roll back catalog second to restore expected dataset view.
3. Validate against golden queries for critical datasets.

## Post-Rollback Validation

- `/readyz` healthy across all pods.
- Dataset open/download failure counters stabilize.
- No increase in circuit-breaker open events.
