# NetworkPolicy Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: map the executable Atlas NetworkPolicy checks to the operator-facing policy contract.

## Covered Checks

- Rendered policies must not fall back to ingress allow-all when NetworkPolicy is enabled.
- `same-namespace` ingress must render a namespace-scoped selector.
- `cluster-aware` egress must render namespace-based dependency rules instead of CIDR-only policy.
- `allowDns=true` must render kube-dns egress.
- metrics-enabled cross-namespace scraping must render a monitoring namespace allowance.
- Production-oriented profiles may not disable network policy without an explicit exception.
- Exception entries must include both owner and expiry.
- CIDR counts may not exceed the configured budget without a relaxed-egress exception.

## Canonical Inputs

- `ops/k8s/charts/bijux-atlas/templates/networkpolicy.yaml`
- `ops/k8s/charts/bijux-atlas/values.schema.json`
- `ops/k8s/values/*.yaml`
- `ops/k8s/examples/networkpolicy/*.yaml`

## Verification

Run the executable contract surface:

```bash
cargo test -p bijux-dev-atlas --test ops_k8s_contracts networkpolicy_
```
