# Ops Naming Rules

- Authority Tier: `machine`
- Audience: `contributors`
## Canonical Terms

- Use `observe` for observability domain naming.
- `obs` is prohibited everywhere, including ids, keys, path segments, and command groups.

## File and Directory Naming

- Use intent-based, durable nouns.
- Avoid timeline or temporary naming.
- Keep schema filenames explicit and stable.

## Generated Naming

- Generated artifacts live under `generated/`, `_generated/`, or `_generated.example/`.
- Generated names must indicate artifact intent (`schema-index`, `readiness-score`, `release-evidence-bundle`).
