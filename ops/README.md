# Ops

Ops policy is enforced by executable contracts, not prose documents.

- Intent: keep `ops/` as operational data + contract surfaces only.
- Contracts runner: `bijux dev atlas contracts ops`.
- Human walkthroughs and procedures live in `docs/operations/`.

## Contract Taxonomy

- Layout contracts: directory and file surface boundaries.
- Schema contracts: JSON/YAML/TOML validity and compatibility.
- Inventory graph contracts: ownership, mapping, and reference integrity.
- Drift contracts: generated vs authored consistency.
- Pins and supply-chain contracts: tool/image/version locks.
- Effect contracts: integration checks executed against runtime environments.
