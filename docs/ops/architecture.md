> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Ops Architecture

Ops uses two sources of truth:
- Machine-verifiable contracts and schemas in `ops/`
- Human guidance and workflows in `docs/ops/` and `docs/operations/`

Core areas:
- Inventory and schema governance: `ops/inventory/`, `ops/schema/`
- Runtime domains: `ops/k8s/`, `ops/observe/`, `ops/load/`, `ops/e2e/`, `ops/datasets/`, `ops/stack/`
- Evidence and generated mirrors: `ops/_generated.example/`
