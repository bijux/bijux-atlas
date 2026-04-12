---
title: bijux-dev-atlas Package
audience: maintainers
type: package
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# bijux-dev-atlas

`bijux-dev-atlas` is the repository control-plane crate for `bijux-atlas`. It
turns governance, documentation, policy validation, report generation, and
operational workflows into a Rust command surface instead of a shell-script
control plane.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| repository governance | checks, audits, invariants, policy loading, security and CI workflows |
| docs and reference tooling | docs validation, builds, reference generation, registry-driven documentation flows |
| report and ops control plane | reports, runtime ops, release, load, perf, and tutorial automation |
| boundary | does not own the product-facing Atlas runtime, dataset behavior, or HTTP API semantics |

## Source Layout

- `crates/bijux-dev-atlas/src/core`
- `crates/bijux-dev-atlas/src/domains`
- `crates/bijux-dev-atlas/src/engine`
- `crates/bijux-dev-atlas/src/registry`
- `crates/bijux-dev-atlas/src/reference`
- `crates/bijux-dev-atlas/src/docs`

## Open Next

- open the [Maintainer Handbook](../../index.md) for maintainer workflows, references, and contracts
- open the [Repository Handbook](../../../repository/index.md) when a maintainer question touches package ownership or repository-wide boundaries
- open [bijux-atlas](../../../runtime/packages/bijux-atlas/index.md) when the issue belongs to product runtime behavior instead of control-plane automation

## Code Anchors

- `crates/bijux-dev-atlas/README.md`
- `crates/bijux-dev-atlas/Cargo.toml`
- `docs/06-development/index.md`
- `docs/07-reference/automation-command-surface.md`
- `docs/08-contracts/automation-contracts.md`

## Review Lens

- maintainer-only automation should stay separate from end-user runtime behavior
- registry, policy, and report surfaces should remain deterministic and auditable
- docs and governance flows should route back to canonical handbook pages instead of creating parallel ad hoc instructions
