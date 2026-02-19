# E2E INDEX

## Purpose
Own composition scenarios across stack + k8s + obs + datasets + load.

## Public entrypoints
- `./ops/run/e2e.sh --suite smoke|k8s-suite|realdata [--fast] [--no-deploy] [--profile kind]`
- `make ops-e2e SUITE=smoke|k8s-suite|realdata`

## Suites
- `ops/e2e/suites/suites.json`
- `ops/e2e/runner/suite.sh --suite <id>`

## Contracts
- `ops/CONTRACT.md`
- `ops/_meta/ownership.json`
