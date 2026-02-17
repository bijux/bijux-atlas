# Runbook: Dataset Corruption

## Symptoms

- integrity re-verification evicts datasets repeatedly
- checksum mismatch errors on dataset open
- elevated `503` for specific dataset ids

## Immediate Actions

1. Pin known-good datasets if available.
2. Block affected dataset from routing if corruption isolated.
3. Trigger dataset re-fetch from source store.

## Investigation

1. Compare manifest sqlite checksum vs local file checksum.
2. Inspect `.verified` marker and manifest version consistency.
3. Verify upstream artifact integrity in store.

## Recovery

1. Purge corrupted cached copy.
2. Re-download and verify checksum.
3. Run smoke query checks for affected dataset.
