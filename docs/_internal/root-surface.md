# Root Surface Rationale

- Owner: `bijux-atlas-operations`
- Audience: `maintainers`
- Stability: `stable`

## Why Root Is Small

A small root keeps contributor onboarding predictable and review diff noise low.
Authority documents and generated references live under `docs/`, `ops/`, or `configs/`.

## What Belongs At Root

- lobby documents: `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CONTRACT.md`, `CHANGELOG.md`, `LICENSE`
- workspace control files: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `Makefile`
- top-level governed directories only

## What Does Not Belong At Root

- generated map/status/progress documents
- checkpoint notes and ad hoc scratch markdown
- duplicate policy roots or alternate registries

## Verification

- Root contracts: `bijux dev atlas contracts root --mode static`
- Repo required suite: `bijux dev atlas check run --suite repo_required --include-internal --include-slow --allow-git`

