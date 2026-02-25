> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Ops Gates Contract

Gates are explicit SSOT definitions in `ops/inventory/gates.json`.

Required gates:
- `ops.doctor`
- `ops.validate`
- `ops.gate.directory-completeness`
- `ops.gate.schema-validation`
- `ops.gate.pin-drift`
- `ops.gate.stack-reproducibility`
- `ops.gate.k8s-determinism`
- `ops.gate.observe-coverage`
- `ops.gate.dataset-lifecycle`
- `ops.gate.unified-readiness`

Each gate must map to a valid action id from `ops/inventory/surfaces.json`.
