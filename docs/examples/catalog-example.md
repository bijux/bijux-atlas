# Catalog Example

- Owner: `bijux-atlas-store`

## What
Reference catalog example used by docs contract checks.

## Why
Provides a compact canonical format for catalog entry expectations.

## Scope
Covers one dataset entry with explicit artifact paths and checksum field.

## Non-goals
Does not define full registry federation behavior.

## Contracts
Catalog examples must remain sorted and deterministic.

## Failure modes
Mismatched catalog shape causes publish and fetch errors.

## How to verify
```bash
$ jq -e '.datasets[0].dataset_id.release == "110"' docs/examples/sample-catalog.json >/dev/null
```

Expected output: command exits `0`.

## See also
- [Reference Index](../reference/index.md)
- [Artifact Schema Contract](../reference/contracts/artifacts/index.md)
- [Contract Examples](../reference/contracts/examples/index.md)
