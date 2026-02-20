# When Layer Drift Fails

Layer drift means the live cluster shape no longer matches `ops/_meta/layer-contract.json`.

Use this order to fix correctly:
1. If chart/rendered resources changed intentionally, update the SSOT contract (`ops/_meta/generate_layer_contract.py`) and commit generated diffs.
2. If drift is unintentional, fix the source layer that owns it (chart/templates or stack setup), not e2e/k8s test fixups.
3. Run `make ops/contract-check` and inspect `layer-drift-triage.json` in the contract-check artifacts.
4. For temporary unavoidable drift, add a scoped exception in `configs/policy/layer-live-diff-allowlist.json` with owner, issue, and expiry.
5. Remove exceptions before expiry and re-run the gate until there are no active differences.

Common triage keys:
- `service.<component>.missing`: expected service absent.
- `service.<component>.port.<name>`: service port mismatch.
- `deployment.<name>.label.<key>`: required label missing.
