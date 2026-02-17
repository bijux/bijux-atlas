# Runbook: DATASET CORRUPTION

- Owner: `bijux-atlas-store`

## Symptoms

- Repeated checksum verification failures.
- Dataset open rejection for specific dataset id.

## Metrics

- `bijux_errors_total`
- `bijux_dataset_misses`
- `bijux_store_open_p95_seconds`

## Commands

```bash
$ cargo run -p bijux-atlas-cli -- atlas dataset validate --deep --dataset release=112,species=homo_sapiens,assembly=GRCh38
$ curl -s http://127.0.0.1:8080/debug/datasets
```

## Expected outputs

- Deep validate reports checksum mismatch for corrupted artifact.
- Debug dataset view marks dataset as unavailable/quarantined.

## Mitigations

- Evict corrupted cache copy.
- Re-fetch artifact and verify manifest lock.

## Rollback

- Serve previous known-good dataset release.
- Freeze publish path until integrity checks pass.

## Postmortem checklist

- Corruption source identified.
- Integrity controls reviewed.
- Additional corruption tests added.
