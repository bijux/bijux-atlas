# Stack INDEX

## Purpose
Own local dependency bring-up and dependency fault APIs.

## Public entrypoints
- `make ops-stack-up PROFILE=kind`
- `make ops-stack-down`

## Suites
- `ops/stack/tests` suites are invoked through higher-level `make ops-k8s-suite` and `make ops-local-full`.

## Contracts
- `ops/CONTRACT.md`
- `ops/_meta/ownership.json`
