# Ops Canonical Layout

The canonical SSOT is `ops/CONTRACT.md`.

## Top-level layout

- `ops/stack/`
- `ops/k8s/`
- `ops/obs/`
- `ops/load/`
- `ops/datasets/`
- `ops/e2e/`
- `ops/_meta/`
- `ops/schema/`
- `ops/_generated/`
- `ops/_artifacts/`

## Rules

- Use Make targets from `ops/INDEX.md`.
- Regenerate committed ops generated outputs with `make ops-gen`.
- Enforce generated drift with `make ops-gen-check`.
- `ops/e2e/` is composition-only.
- `ops/e2e/k8s/tests/` keeps wrapper entrypoints only; invariant tests live in `ops/k8s/tests/`.
- Operator entrypoints live in `atlasctl ops ...`; shared helper assets live under atlasctl ops runtime/k8s test asset paths, and domain-local helpers remain under `ops/*/scripts/`.
- No symlinked domain directories under `ops/`.
- Artifacts write to `ops/_artifacts/` unless allowlisted in `configs/ops/artifacts-allowlist.txt`.
- Empty dirs must contain `INDEX.md` explaining why they exist.
- Fault injection calls must go through `./bin/atlasctl ops kind fault <name>`.
