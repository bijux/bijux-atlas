# When Layer Drift Fails

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Layer drift means the live cluster shape no longer matches `ops/inventory/layers.json`.

## What

Defines the required triage order when live k8s state diverges from the layer contract.

## Why

Ensures fixes are made in the owning layer instead of masked by downstream fixups.

## Contracts

Use this order to fix correctly:
1. If chart/rendered resources changed intentionally, update the SSOT contract (`crates/bijux-dev-atlas/src/commands/ops/meta/generate_layer_contract.py`) and commit generated diffs.
2. If drift is unintentional, fix the source layer that owns it (chart/templates or stack setup), not e2e/k8s test fixups.
3. Run `make ops/contract-check` and inspect `layer-drift-triage.json` in the contract-check artifacts.
4. For unavoidable drift with explicit expiry, add a scoped exception in `configs/policy/layer-live-diff-allowlist.json` with owner, issue, and expiry.
5. Remove exceptions before expiry and re-run the gate until there are no active differences.

Common triage keys:
- `service.<component>.missing`: expected service absent.
- `service.<component>.port.<name>`: service port mismatch.
- `deployment.<name>.label.<key>`: required label missing.

Chart values anchors to check before changing contracts:
- `values.service`: service names, selectors, and exposed ports.
- `values.server`: container ports and runtime settings that map to service targets.
- `values.rollout`: pod template label/annotation behavior that must remain contract-compatible.

## Failure modes

Patching live resources in tests, adding broad allowlist entries, or changing contracts without source-layer updates causes repeated drift failures.

## How to verify

```bash
make ops/contract-check
```

Expected output: live snapshot comparison passes or reports only approved allowlist differences.

## See also

- `docs/architecture/layering/BOUNDARY_RULES.md`
- `docs/operations/e2e/what-e2e-is-not.md`
