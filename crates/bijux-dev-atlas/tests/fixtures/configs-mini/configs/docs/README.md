# Docs Configs

Canonical docs toolchain and lint configuration.

Reference index: `configs/docs/README.md`

## Files

- `.vale.ini`
  - Consumer: `vale` in docs CI and local `make docs`.
- `.vale/styles/**`
  - Consumer: `vale` style/terminology rules.
- `requirements.txt`
  - Consumer: local dev dependency baseline for docs.
- `requirements.lock.txt`
  - Consumer: pinned reproducible docs env in `make docs` / `make docs-serve`.

## Vale Rules

- Base style: `Bijux`
- Terminology rule enforces canonical terms (`SSOT`, `Release-indexed`, `dataset`).

## Verification

```bash
make docs
```
