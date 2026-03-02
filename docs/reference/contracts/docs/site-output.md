# Docs Site Output

- Owner: `docs-governance`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: define the single build-output closure for published documentation.

## Checks

- `DOCS-SITE-001`: the configured `site_dir` must exist after `mkdocs build --strict`.
- `DOCS-SITE-002`: the configured `site_dir` must contain `index.html`.
- `DOCS-SITE-003`: the configured `site_dir` must contain the configured assets directory.
- `DOCS-SITE-FILE-COUNT`: the configured `site_dir` must contain at least the minimum file count from `configs/docs/site-output-contract.json`.

## Reproduce locally

- `bijux dev atlas docs build --allow-subprocess --allow-write --format json`
- `bijux dev atlas docs site-dir --format json`
- `bijux dev atlas contracts repo --mode effect --allow-subprocess --filter-contract REPO-005`

## Configuration

- `mkdocs.yml` is the source of truth for `docs_dir` and `site_dir`.
- `configs/docs/site-output-contract.json` sets the minimum file count and assets directory name.
- `configs/contracts/docs-site-output.schema.json` defines the config shape that the control-plane reads.

## Failure triage

- If `DOCS-SITE-001` fails, the docs build did not materialize the configured output directory.
- If `DOCS-SITE-002` fails, the build is incomplete or wrote to the wrong location.
- If `DOCS-SITE-003` fails, static assets were not emitted where the workflow expects them.
- If `DOCS-SITE-FILE-COUNT` fails, the build likely produced an empty or partial site.
