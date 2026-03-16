---
title: Command Surface Reference
audience: operators
type: reference
status: generated
owner: bijux-atlas-operations
last_reviewed: 2026-03-16
---

# Command Surface Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Stability: `stable`
- Source-of-truth: `bijux dev atlas --help`, `bijux dev atlas ops --help`, `docs/_internal/generated/makes-targets.md`

## Purpose

Generated reference for the supported command surface. Narrative docs should link here instead of restating command lists.

## bijux-dev-atlas

```text
Bijux Atlas development control-plane

Usage: bijux-dev-atlas [OPTIONS] [COMMAND]

Commands:
  ops
  docs
  reports
  configs
  governance
  system
  audit
  observe
  api
  load
  invariants
  security
  check
  checks
  runtime
  tutorials
  migrations
  datasets
  ingest
  perf
  policies
  ci
  registry
  suites
  tests
  list
  describe
  run
  validate
```

## bijux-dev-atlas ops

```text
Usage: bijux-dev-atlas ops [OPTIONS] <COMMAND>

Commands:
  logs
  describe
  events
  resources
  kind
  helm
  list
  explain
  stack
  k8s
  profiles
  profile
  load
  datasets
  e2e
  scenario
  obs
  schema
  inventory-domain
  report-domain
  evidence
  diagnose
  drills
  tools
  suite
  doctor
  validate
  graph
  inventory
  docs
  docs-verify
  conformance
  report
  helm-env
  readiness
  render
  install
  smoke
  status
  list-profiles
  explain-profile
  list-tools
  verify-tools
  list-actions
  plan
  package
  release-plan
  install-plan
  up
  down
  clean
  cleanup
  reset
  pins
  generate
```

## Make Wrapper Surface

See `docs/_internal/generated/makes-targets.md` and generated ops surface references. Narrative docs must not duplicate long `make ops-*` command lists.

## Regenerate

- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`

## Stability

This page is generated from the checked-in command surface and should remain aligned with the current control plane.
