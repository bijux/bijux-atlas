# Profiles

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@f585bb97e56a5d8adfd1b3d7c557a39d0dd9c8cb`
- Reason to exist: define the supported Kubernetes profile intents and the controls each one must keep.

## Purpose

Atlas profiles are named operational intents. The chart defaults remain the shared substrate, while
each profile overlay in `ops/k8s/values/` captures the toggles that must change for one specific
environment.

## Profiles

### CI

Purpose: fast validation with cached-only startup and no external egress.

What can break: enabling catalog-dependent readiness or external dependency egress makes CI slow
and nondeterministic.

Rollback: restore `server.cachedOnlyMode=true`, `server.readinessRequiresCatalog=false`, and keep
HPA disabled.

### Kind

Purpose: realistic local cluster install with dependency-aware egress and no persistent volumes.

What can break: changing dependency namespaces or enabling persistence introduces local cluster
drift.

Rollback: keep `cache.pvc.enabled=false`, reset network policy to the named `atlas-deps`
namespace, and redeploy.

### Offline

Purpose: disconnected startup path that prewarms one governed dataset and serves from cache only.

What can break: removing the pinned dataset or disabling init prewarm leaves readiness without the
required local data.

Rollback: restore the pinned dataset entry and `cache.initPrewarm.enabled=true`, then redeploy.

### Performance

Purpose: load-focused validation with digest-pinned image, metrics, and autoscaling.

What can break: reintroducing `image.tag`, disabling metrics, or removing HPA turns the profile
into something other than a performance gate.

Rollback: restore the digest-pinned image, metrics service monitor, and HPA settings.

### Production

Purpose: standard production rollout with dependency-aware network policy and autoscaling.

What can break: enabling debug endpoints or relaxing network policy makes the profile unsafe for
shared clusters.

Rollback: restore cluster-aware egress, HPA, and the production safety toggles recorded in
`ops/k8s/values/profiles.json`.

### Production Minimal

Purpose: minimum production-safe footprint with digest pinning, HPA, and cluster-aware egress.

What can break: removing digest pinning or disabling HPA drops the profile below the production
safety baseline.

Rollback: restore the immutable image digest and HPA settings.

### Production HA

Purpose: high-availability production rollout with multiple replicas, PDB, and stricter probes.

What can break: reducing replicas below two or loosening disruption controls undermines HA goals.

Rollback: restore replica count, PDB, and probe timings from the profile file.

### Production Airgap

Purpose: disconnected production install that uses a local registry and prewarmed pinned assets.

What can break: using remote image references or removing pinned datasets defeats the air-gap
contract.

Rollback: restore the local registry image digest and the governed pinned dataset list.

## Verify

```bash
helm lint ops/k8s/charts/bijux-atlas -f ops/k8s/values/prod-minimal.yaml
helm lint ops/k8s/charts/bijux-atlas -f ops/k8s/values/prod-ha.yaml
helm lint ops/k8s/charts/bijux-atlas -f ops/k8s/values/prod-airgap.yaml
```

## Rollback

Reapply the last known-good profile overlay and confirm the resulting toggles still match
`ops/k8s/values/profiles.json` before you retry the rollout.

## Related

- [Profile Change Policy](profile-change-policy.md)
- [Profile Ownership](profile-ownership.md)
