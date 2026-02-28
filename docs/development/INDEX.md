# Development Index

- Owner: `docs-governance`
- Stability: `stable`

## What

Index page for `development` documentation.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

List and maintain links to section pages in this directory.

- [Makefiles Surface](makefiles/surface.md)
- [Make Targets](make-targets.md)
- [Control Plane Commands](tooling/control-plane.md)
- [Tooling Index](tooling/INDEX.md)
- [Script Naming](script-naming.md)
- [Docs Depth Contract](docs-depth-contract.md)
- [Repository Layout](repo-layout.md)
- [Repository Surface](repo-surface.md)
- [Symlink Index](symlinks.md)
- [Tool Config Shims](tool-config-shims.md)
- [Config Versioning](config-versioning.md)
- [Local Noise Policy](local-noise.md)
- [Contributing](contributing.md)
- [Dataset Identity Contract](dataset-identity.md)
- [Normalized Format (Debug)](normalized-format.md)
- [Add Dataset Type](add-dataset-type.md)
- [Add Endpoint](add-endpoint.md)
- [Add Metric or Span](add-metric-span.md)

## Failure modes

Missing index links create orphan docs.

## How to verify

```bash
$ make docs
```

Expected output: docs build and docs-structure checks pass.

## See also

- [Docs Home](../index.md)
- [Naming Standard](../_style/naming-standard.md)
- [Terms Glossary](../_style/terms-glossary.md)
