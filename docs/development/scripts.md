# Scripts Governance

- Owner: `platform`

## What

Repository script taxonomy, contracts, and contribution rules.

## Taxonomy

- `scripts/areas/public/`: only make-callable script entrypoints.
- `scripts/areas/internal/`: helper scripts called by public wrappers.
- `scripts/areas/dev/`: local-only helpers.
- `scripts/areas/tools/`: shared Python helper modules.
- Domain buckets allowed for cross-cutting workflows: `scripts/areas/docs/`, `scripts/areas/contracts/`, `scripts/areas/layout/`, `scripts/areas/fixtures/`, `scripts/areas/release/`, `scripts/areas/ops/`, `scripts/areas/bootstrap/`, `scripts/bin/`.

## Contracts

- Every script must include shebang + `Purpose`, `Inputs`, `Outputs` header lines.
- Public scripts must also declare: `owner`, `purpose`, `stability`, `called-by`.
- Scripts must not assume implicit cwd; resolve repo root explicitly.
- Public wrappers should stay thin; move reusable logic into `scripts/areas/internal/` or `scripts/areas/tools/`.
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
