# K8s INDEX

## Purpose
Own Helm chart install profiles and Kubernetes-only gates.

## Public entrypoints
- `make ops-k8s-suite PROFILE=kind SUITE=<suite-id>`

## Suites
- `ops/k8s/tests/suites.json`
- Runner: `bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/run_suite.py --suite <suite-id>`
- Generated test surface: `ops/k8s/tests/INDEX.md`

## Contracts
- `ops/CONTRACT.md`
- `ops/_meta/ownership.json`
