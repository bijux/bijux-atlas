# Config Compiler Contract

## Authority

- Authoritative inventory home: `ops/inventory/` (not `configs/inventory/`).
- `configs/` holds policy/config inputs and compiler outputs, not duplicated ops inventory SSOTs.

## Compiler API

- `atlasctl config validate`: validate config files, schema pairs, overlay merge model, and lock discipline.
- `atlasctl config gen`: render deterministic compiler outputs under `configs/_generated/`.
- `atlasctl config diff --fail`: drift gate for generated compiler outputs.
- `atlasctl config fmt`: canonicalize JSON formatting for `configs/**`.

## Overlay Model

- Base: `configs/ops/env.schema.json`
- Overlays: `ops/env/overlays/<name>.yaml|json`
- Allowed override fields: `default`, `description`

## Output Discipline

- `configs/_generated/*` files are generated-only (header + checksum enforced).
- Human-facing docs/workflows/makefiles must not reference `configs/_generated/` directly.
- Use `atlasctl config ...` commands as the stable interface.
