# Input Sources

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `ops`
- Stability: `evolving`

## Why

Ingest must capture source provenance and avoid unsafe TOCTOU drift between fetch and execution.

## Make-first usage

Use make workflows for ops runs:

```bash
make ops-publish DATASET=medium
```

## CLI contract

`bijux-atlas` supports explicit input verification before ingest:

```bash
cargo run -p bijux-atlas-cli -- atlas ingest-verify-inputs \
  --gff3 ./ops/datasets/fixtures/medium/v1/data/genes.gff3 \
  --fasta ./ops/datasets/fixtures/medium/v1/data/genome.fa \
  --fai ./ops/datasets/fixtures/medium/v1/data/genome.fa.fai \
  --output-root ./artifacts/e2e-datasets
```

Supported sources:

- local path or `file://...`
- `http://...` and `https://...` (requires `--allow-network-inputs`)
- `s3://bucket/key` via `ATLAS_S3_ENDPOINT` (requires `--allow-network-inputs`)

## Lockfile and resume

Input verification writes `output_root/_ingest_inputs/inputs.lock.json` with:

- original source
- resolved URL
- sha256 checksum
- expected size
- output path

`--resume` enforces lockfile TOCTOU checks and fails on hash/size mismatch.

## Deterministic fetch/decompress

- Download path contract: temp write -> verify -> atomic move.
- `.gz` and `.zst` sources are decompressed deterministically to stable local files.
- Corrupt/stale fetches are quarantined under `output_root/_ingest_inputs/quarantine/`.

## Failure modes

- Network sources are denied unless explicitly allowed.
- Resume with mismatched lock entries fails.
- Corrupt download verification fails and quarantines temp artifacts.
