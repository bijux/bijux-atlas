# Fixture Policy

CI fixtures:

- `crates/bijux-atlas-ingest/tests/fixtures/minimal`
- `crates/bijux-atlas-ingest/tests/fixtures/edgecases`
- `crates/bijux-atlas-ingest/tests/fixtures/contigs`

Non-CI fixture:

- medium dataset is fetched via `make fetch-fixtures` using pinned URL + sha256 in `fixtures/medium/manifest.lock`.

Golden query snapshots:

- Query definitions: `fixtures/medium/golden_queries.json`.
- Snapshot output (manual): `fixtures/medium/golden_snapshot.json`.
