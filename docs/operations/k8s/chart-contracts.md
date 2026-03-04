# Chart Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the Kubernetes chart invariants that must remain true across profiles and releases.

## Install safety invariants

- `server.cachedOnlyMode=true` requires `server.readinessRequiresCatalog=false`.
- `cache.initPrewarm.enabled=true` requires `cache.pinnedDatasets` to contain at least one dataset identifier.
- `image.digest` requires `image.tag` to be explicitly empty.
- `hpa.enabled=true` requires `metrics.customMetrics.enabled=true` and `serviceMonitor.enabled=true`.
- `alertRules.enabled=true` requires `serviceMonitor.enabled=true`.

## Runtime mapping invariants

- The chart emits only canonical runtime env keys consumed by `atlas-server`.
- The chart does not emit `ATLAS_DEV_ALLOW_UNKNOWN_ENV`.
- Store endpoints are populated only from `store.endpoint` and `store.s3PresignedBaseUrl`.

## Network policy invariants

- The default chart policy includes both ingress and egress controls.
- DNS egress is controlled only by `networkPolicy.allowDns`.
- Additional egress CIDRs must be listed explicitly in `networkPolicy.allowedEgressCIDRs`.

## How to verify

- `helm lint ops/k8s/charts/bijux-atlas`
- `helm template atlas ops/k8s/charts/bijux-atlas`
- `bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-PROFILES-001`

## See also

- [Profile Invariants](profile-invariants.md)
- [Helm Env Allowlist Subset](../../reference/contracts/ops/helm-env-subset.md)
