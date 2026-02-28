# Ops Entrypoints

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `docs/reference/commands.md`, `docs/reference/configs.md`, `ops/inventory/surfaces.json`

## Canonical Entrypoints

- Primary CLI: `bijux dev atlas ops ...`
- `make` wrappers are thin convenience wrappers and should not be treated as the authoritative command surface.
- For current command lists and wrappers, use the generated references below.

## References

- [Command Surface Reference](reference/commands.md)
- [Ops Surface Reference](reference/ops-surface.md)
- [Ops Filesystem Layout](ops-layout.md)

## Rules

- Narrative docs must link to generated command references instead of embedding long command lists.
- Do not instruct operators to edit generated directories under `ops/_generated*` directly.

Related contracts: OPS-ROOT-023, OPS-ROOT-017.
