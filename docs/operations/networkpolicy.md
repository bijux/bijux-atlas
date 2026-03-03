# NetworkPolicy

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define the canonical Atlas NetworkPolicy modes and the namespace contract they rely on.

Related ops contracts: `OPS-ROOT-023`, `OPS-K8S-001`.

## Purpose

Use one explicit NetworkPolicy model for Atlas so operators can reason about what traffic is
permitted, which dependencies are assumed, and what must be labeled before a release goes live.

The canonical intent is east-west isolation first, with optional internet egress limiting layered
on top. Use `networkPolicy.mode` for the primary egress selector and `networkPolicy.ingressMode`
for the primary ingress selector; the nested `networkPolicy.egress.mode` and
`networkPolicy.ingress.mode` keys remain valid as compatibility aliases.

## Modes

- `disabled`: render no policy and rely on the cluster default posture.
- `internet-only`: allow DNS plus outbound TCP `80/443` only to the explicit CIDRs in
  `networkPolicy.egress.allowCidrs`.
- `cluster-aware`: allow DNS plus outbound traffic only to explicitly named dependency namespaces,
  using `namespaceSelector` instead of hardcoded service CIDRs.
- `custom`: reserve the chart surface but emit no automatic ingress or egress rules; operators must
  layer their own policy objects.

Ingress modes:

- `disabled`: emit no ingress rules.
- `same-namespace`: allow only in-namespace callers and deny cross-namespace ingress.
- `selected`: allow only the namespaces listed in `networkPolicy.ingress.allowNamespaces`.
- `custom`: emit no automatic ingress rules and require operator-supplied overlays.

Compatibility aliases:

- `networkPolicy.mode` mirrors the primary egress selector.
- `networkPolicy.ingressMode` mirrors the primary ingress selector.
- `networkPolicy.monitoring.allowNamespace` mirrors `networkPolicy.allowMonitoringNamespace`.
- `networkPolicy.dependencies.otel` mirrors `networkPolicy.dependencies.otelCollector`.

## Dependency Matrix

| Dependency | Values Key | Default Ports | Namespace Allowlist |
| --- | --- | --- | --- |
| Redis | `networkPolicy.dependencies.redis` | `6379` | `networkPolicy.egress.allowNamespaces` |
| MinIO | `networkPolicy.dependencies.minio` | `9000` | `networkPolicy.egress.allowNamespaces` |
| Catalog HTTP | `networkPolicy.dependencies.catalog` | `80`, `443` | `networkPolicy.egress.allowNamespaces` |
| OTel Collector | `networkPolicy.dependencies.otelCollector` | `4317`, `4318` | `networkPolicy.egress.allowNamespaces` |

For kind and other low-friction local clusters, the minimum practical posture is:

- egress: `internet-only`
- ingress: `same-namespace`
- monitoring namespace: explicit only when metrics scraping is enabled across namespaces

## Namespace Label Contract

Cluster-aware mode requires dependency namespaces to be labeled consistently. The canonical label
contract is:

- `bijux.atlas/dependency=true`

Atlas selects dependency namespaces by name today, but the operational contract is that dependency
namespaces also carry the label above so future policy tightening can move to label-based
selection without reclassifying namespaces.

## Operator Warning

`cluster-aware` requires namespace labels and explicit namespace allowlists. If those are wrong,
Atlas will fail by policy rather than silently falling back to broad egress.

Atlas does not require in-cluster HTTPS access to the Kubernetes API for normal operation, so the
chart keeps kube-apiserver egress closed by default.

## Monitoring Namespace

When `metrics.enabled=true`, the chart adds an ingress allowance for
`networkPolicy.allowMonitoringNamespace` so Prometheus scraping can continue without opening
cross-namespace application traffic.

Example ServiceMonitor placement:

```yaml
networkPolicy:
  allowMonitoringNamespace: observability
```

Run the ServiceMonitor and Prometheus Operator in the `observability` namespace, or set
`networkPolicy.allowMonitoringNamespace` to the namespace you actually use.

## Compatibility

NetworkPolicy enforcement depends on the active CNI plugin. Validate behavior on the same CNI that
will run production before treating a rendered policy as sufficient evidence.

## Verify

```bash
helm lint ops/k8s/charts/bijux-atlas -f ops/k8s/values/prod.yaml
helm template atlas ops/k8s/charts/bijux-atlas -f ops/k8s/values/prod.yaml --show-only templates/networkpolicy.yaml
```

Expected result: the rendered policy contains only the selected ingress mode and the dependency
ports enabled for the profile.

## Rollback

If a policy blocks expected traffic, switch the affected profile to a narrower known-good posture
(`disabled` or `internet-only`) and redeploy while you fix the namespace contract.

## Production Recommendation

Use `cluster-aware` egress with an explicit dependency namespace allowlist and keep ingress at
`same-namespace` unless you have a separate ingress or observability namespace that must be named
explicitly.

## Troubleshooting

- DNS failures usually mean `allowDns=false` or a blocked kube-dns rule.
- Dependency timeouts in `cluster-aware` mode usually mean a missing namespace entry in
  `networkPolicy.egress.allowNamespaces`.
- Unexpected cross-namespace ingress means the wrong ingress mode was selected for the profile.
