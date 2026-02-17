# Non Goals

- Owner: `bijux-atlas-product`

## What

Strict exclusions for v1/v2 scope control.

## Why

Prevents accidental expansion and contract instability.

## Scope

Applies to API, ingest, and operations commitments.

## Non-goals

- Atlas will not perform variant interpretation.
- Atlas will not mutate published datasets.
- Atlas will not accept implicit dataset defaults.
- Atlas will not execute remote code/plugins from requests.
- Atlas will not provide write APIs for genomic entities.

## Contracts

Requests requiring excluded behavior must return explicit rejection errors.

## Failure modes

Ambiguous scope introduces unstable endpoints and untestable behavior.

## How to verify

```bash
$ rg -n "implicit default|write endpoint|mutation" docs/product docs/reference
```

Expected output: exclusions are explicit and no conflicting commitments exist.

## See also

- [What Is Atlas](WHAT_IS_ATLAS.md)
- [Compatibility Promise](COMPATIBILITY_PROMISE.md)
- [Reference Grade Checklist](REFERENCE_GRADE_ACCEPTANCE_CHECKLIST.md)
