# atlasctl Deletion PR Checklist

Use this checklist for the final removal PR that deletes `packages/atlasctl/`.

## Required

- `packages/atlasctl/` is deleted.
- No `./bin/atlasctl` shim exists.
- `makefiles/` has zero `atlasctl` references.
- `.github/workflows/` has zero `atlasctl` references.
- `configs/` has zero `atlasctl` references (except explicit historical tombstones if any).
- Public docs no longer describe `atlasctl` as the control plane.
- `mkdocs.yml` public nav does not link to `atlasctl` operator pages.
- CI passes Rust control-plane lanes (`check`, `ops`, `docs`, `configs`).

## Artifacts and Reports

- No CI job writes `artifacts/reports/atlasctl/`.
- Replacement reports (if needed) write under `artifacts/reports/dev-atlas/`.

## Lock-the-door

- Repo checks fail on new `atlasctl` references outside `docs/tombstones/atlasctl/`.
- Repo checks fail if `packages/atlasctl/` or `bin/atlasctl` reappear.
