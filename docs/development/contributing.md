# Contributing

- Owner: `docs-governance`

## Layout rule

Do not add new root directories. Put operational assets under `ops/` and expose workflows through `make` targets.

## Required checks

```bash
$ make atlasctl-check-layout
$ make governance-check
```

## See also

- [Repository Layout](repo-layout.md)
- [Ops Canonical Layout](ops-canonical-layout.md)
