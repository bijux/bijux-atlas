# Fixture Lifecycle

Fixtures are versioned release artifacts.

1. Add or update fixture content under `ops/datasets/fixtures/<family>/vN/`.
2. Produce a tarball in `assets/` and set `archive=` + `sha256=` in `manifest.lock`.
3. Keep `src/` input copies and query/response golden files in the same version directory.
4. Refresh `ops/datasets/generated/fixture-inventory.json` with all versions and asset hashes.
5. Update `ops/e2e/fixtures/fixtures.lock` hash pins when fixture inventory or allowlist changes.
6. Regenerate `ops/_generated.example/fixture-drift-report.json` and keep status clean.
