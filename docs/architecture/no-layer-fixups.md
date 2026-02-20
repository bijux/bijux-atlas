# No Layer Fixups Addendum

- Owner: `platform-architecture`
- Stability: `stable`

## Rule

The scripting layer is an orchestrator and must never patch lower layers to mask defects.

## Enforced boundaries

- Scripting commands cannot perform cross-layer fixups (for example e2e patching k8s deployment state).
- Fixes must be applied in the owning layer (`stack`, `k8s`, `e2e`, `obs`, or `load`) and then consumed by scripting.
- Reports may describe failures, but report generation must not mutate runtime state to force green results.

## Verification

```bash
make policies/boundaries-check
make ops/contract-check
```

See also:
- [Layering No Layer Fixups](layering/no-layer-fixups.md)
- [Scripting Architecture Contract](scripting.md)
