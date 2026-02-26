# Air-Gapped Install and Artifact Mirroring

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Goal
Run Atlas without direct internet/store access by mirroring deterministic artifact packs.

## Produce pack
For each dataset:

```bash
bijux-atlas atlas dataset pack \
  --root /path/to/artifacts \
  --release 110 --species homo_sapiens --assembly GRCh38 \
  --out artifacts/packs/110_homo_sapiens_GRCh38.tar
```

## Verify pack integrity

```bash
bijux-atlas atlas dataset verify-pack --pack artifacts/packs/110_homo_sapiens_GRCh38.tar
```

This checks `manifest.lock` against `manifest.json` and `gene_summary.sqlite`.

## Mirror and unpack
- Copy packs into the offline environment.
- Extract into offline store root preserving file names.
- Publish/update offline `catalog.json` using `atlas catalog publish`.

## Offline serving
- Enable cached-only profile / mode.
- Ensure pinned datasets are preloaded.
- Readiness remains healthy in cached-only mode even if store endpoints are unreachable.

## See also

- `ops-ci`
