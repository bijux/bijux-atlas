# Scripts Governance

- Owner: `platform`

## What

Repository script taxonomy, contracts, and contribution rules.

## Taxonomy

- `scripts/public/`: only make-callable script entrypoints.
- `scripts/internal/`: helper scripts called by public wrappers.
- `scripts/dev/`: local-only helpers.
- `scripts/tools/`: shared Python helper modules.
- Domain buckets allowed for cross-cutting workflows: `scripts/docs/`, `scripts/contracts/`, `scripts/layout/`, `scripts/fixtures/`, `scripts/release/`, `scripts/ops/`, `scripts/bootstrap/`, `scripts/bin/`.

## Contracts

- Every script must include shebang + `Purpose`, `Inputs`, `Outputs` header lines.
- Public scripts must also declare: `owner`, `purpose`, `stability`, `called-by`.
- Scripts must not assume implicit cwd; resolve repo root explicitly.
- Public wrappers should stay thin; move reusable logic into `scripts/internal/` or `scripts/tools/`.
- Ops shared shell helpers are canonical in `ops/_lib/`.

## Naming

- Use kebab-case and verb-noun naming for shell entrypoints (example: `check-layout.sh`, `generate-report.sh`).
- Python modules use snake_case filenames.

## Verification

```bash
make scripts-audit
make scripts-lint
make scripts-test
make scripts-graph
```
