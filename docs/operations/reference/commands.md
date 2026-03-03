# Command Surface Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `bijux dev atlas --help`, `bijux dev atlas ops --help`, `docs/_internal/generated/make-targets.md`

## Purpose

Generated reference for the supported command surface. Narrative docs should link here instead of restating command lists.

## bijux-dev-atlas

```text
Bijux Atlas development control-plane

Usage: bijux-dev-atlas [OPTIONS] [COMMAND]

Commands:
  ops
  docs
  demo
  configs
  governance
  security
  datasets
  ingest
  perf
  policies
  ci
  check
  contract
  registry
  suites
  validate
```

## bijux-dev-atlas ops

```text
Usage: bijux-dev-atlas ops <COMMAND>

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
  load
  datasets
  e2e
  obs
  schema
  inventory-domain
  report-domain
  evidence
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
  up
  down
  clean
  cleanup
  reset
  pins
  generate
```

## Make Wrapper Surface

See `docs/_internal/generated/make-targets.md` and generated ops surface references. Narrative docs must not duplicate long `make ops-*` command lists.

## Regenerate

- `bijux dev atlas docs reference generate --allow-subprocess --allow-write`
