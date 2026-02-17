# Makefiles SSOT

- Owner: `build-and-release`

## What

Defines the single-source build and operations target surface for the repository.

## Why

Keeps operational entrypoints stable, discoverable, and auditable through `make`.

## Contracts

- Root `Makefile` is a thin dispatcher that only includes `makefiles/*.mk`.
- Public interfaces are make targets, not direct script paths.
- Domain logic lives in:
  - `makefiles/root.mk`
  - `makefiles/cargo.mk`
  - `makefiles/cargo-dev.mk`
  - `makefiles/docs.mk`
  - `makefiles/ops.mk`
  - `makefiles/policies.mk`
- Any new public target must be listed in `docs/development/makefiles/surface.md`.

## Failure modes

- Direct script usage bypasses target contracts and drifts from CI behavior.
- Untracked target additions create undocumented and unstable interfaces.

## How to verify

```bash
$ make help
$ python3 scripts/docs/check_make_targets_documented.py
$ make ops-script-coverage
```

Expected output: target listing, target docs check, and ops script mapping check pass.

## See also

- [Makefiles Public Surface](../docs/development/makefiles/surface.md)
- [Repository Surface](../docs/development/repo-surface.md)
