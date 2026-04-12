---
title: Automation Command Surface
audience: maintainer
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Automation Command Surface

This page summarizes the current maintainer command families and the stable wrapper entrypoints around them.

The installed maintainer namespace is `bijux dev atlas ...`.
The direct binary remains `bijux-dev-atlas`.

## Primary Entrypoints

- `bijux dev atlas`: the canonical installed repository automation namespace
- `bijux-dev-atlas`: the direct repository automation binary
- `make ci-fast`: fast feedback lane wrapper
- `make ci-pr`: pull-request lane wrapper
- `make ci-nightly`: broader nightly lane wrapper
- `make docs-build`: docs build wrapper

These entrypoints are listed together because Atlas treats them as one maintainer-facing automation
surface, with `bijux dev atlas` as the canonical root and `make` as the thin convenience wrapper.

## Global Options

The top-level binary exposes common flags that show up across most command families:

- `--output-format human|json|both` for renderer selection
- `--json`, `--quiet`, `--verbose`, and `--debug` for output control
- `--repo-root <path>` when a command needs an explicit workspace root
- `--no-deprecation-warn` when you need quiet automation output

## Discovery Commands

Use these commands when you need to inspect what the control plane knows before you execute anything:

```bash
bijux dev atlas list --format json
bijux dev atlas check list
cargo run -q -p bijux-dev-atlas -- list --format json
cargo run -q -p bijux-dev-atlas -- describe --help
cargo run -q -p bijux-dev-atlas -- check list
cargo run -q -p bijux-dev-atlas -- suites list --format json
```

## Main Command Families

- `check`: list, explain, run, or doctor individual checks
- `suites`: list, run, describe, diff, and inspect grouped execution lanes
- `reports`: list governed report families, build indexes, inspect progress, and validate report directories
- `docs`: validate, build, serve, lint, inventory, reference, generate, and redirect docs artifacts
- `governance`: inspect rules, validate governance state, check doctrine, inspect deprecations, and build ADR indexes
- `validate`, `run`, and `list`: generic execution and discovery helpers for registry-backed surfaces
- domain families such as `ops`, `security`, `perf`, `tests`, `tutorials`, `configs`, and `registry` expose narrower workflows in their area

## Selection and Execution Flags

The most important execution selectors are:

- `--suite <name>` for suite-backed selection
- `--tag <tag>`, `--domain <domain>`, `--id <id>`, and `--name <name>` for focused check execution
- `--mode static|effect` on `check run`
- `--mode pure|effect|all` on `suites run`
- `--format text|json|jsonl` when the command owns a machine-readable report format

## Capability Flags

Effectful commands fail closed unless the required capability is explicitly allowed.

- `--allow-subprocess` for shelling out to external tools
- `--allow-write` for generating or updating artifacts
- `--allow-network` for network-dependent commands
- `--allow-git` for commands that need explicit git access

## Current Suite and Report Anchors

As of March 15, 2026, the stable suite ids exposed by `suites list --format json` are:

- `checks`
- `contracts`
- `tests`

The current top-level `reports` catalog includes report families such as `closure-index`, `docs-build-closure-summary`, `docs-site-output`, `helm-env`, and `ops-profiles`.

## Related Pages

- [Automation Reports Reference](automation-reports-reference.md)
- [Automation Control Plane](automation-control-plane.md)
- [Automation Contracts](../governance/automation-contracts.md)

## Purpose

This page is the lookup reference for automation command surface. Use it when you need the current checked-in surface quickly and without extra narrative.

## Stability

This page is a checked-in reference surface. Keep it synchronized with the repository state and generated evidence it summarizes.
