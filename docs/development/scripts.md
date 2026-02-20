# Scripts Governance

- Owner: `platform`

## What

Repository script taxonomy, contracts, and contribution rules.

## Taxonomy

- `scripts/areas/public/`: only make-callable script entrypoints.
- Domain buckets are exposed through `atlasctl` command groups (`docs`, `contracts`, `layout`, `fixtures`, `release`, `ops`, `bootstrap`) and thin `bin/` shims only.

## Contracts

- Every script must include shebang + `Purpose`, `Inputs`, `Outputs` header lines.
- Public scripts must also declare: `owner`, `purpose`, `stability`, `called-by`.
- Scripts must not assume implicit cwd; resolve repo root explicitly.
- Public wrappers should stay thin; move reusable logic into Python modules (`packages/atlasctl/src`) or `ops/_lib/`.
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
