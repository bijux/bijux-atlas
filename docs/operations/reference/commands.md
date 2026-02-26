# Command Surface Reference

- Owner: `bijux-atlas-operations`
- Tier: `generated`
- Audience: `operators`
- Source-of-truth: `bijux dev atlas --help`, `bijux dev atlas ops --help`, `makefiles/GENERATED_TARGETS.md`

## Purpose

Generated reference for the supported command surface. Narrative docs should link here instead of restating command lists.

## bijux-dev-atlas

```text
Bijux Atlas development control-plane

Usage: bijux-dev-atlas [OPTIONS] [COMMAND]

Commands:
  ops
  docs
  configs
  policies
  check
```

## bijux-dev-atlas ops

```text
Usage: bijux-dev-atlas ops <COMMAND>

Commands:
  list
  explain
  stack
  k8s
  load
  e2e
  obs
  tools
  suite
  doctor
  validate
  inventory
  docs
  conformance
  report
  render
  install
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

See `makefiles/GENERATED_TARGETS.md` and generated ops surface references. Narrative docs must not duplicate long `make ops-*` command lists.

## Regenerate

- `bijux dev atlas --help`
- `bijux dev atlas ops --help`
- `bijux dev atlas docs generate` (planned)
