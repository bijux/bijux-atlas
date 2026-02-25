# Ops Naming Rules

## Canonical Terms

- Use `observe` for observability domain naming.
- `obs` naming is retired and forbidden in new paths, filenames, and contract keys.

## File and Directory Naming

- Use intent-based, durable nouns.
- Avoid timeline or temporary naming.
- Keep schema filenames explicit and stable.

## Generated Naming

- Generated artifacts live under `generated/`, `_generated/`, or `_generated.example/`.
- Generated names must indicate artifact intent (`schema-index`, `readiness-score`, `release-evidence-bundle`).
