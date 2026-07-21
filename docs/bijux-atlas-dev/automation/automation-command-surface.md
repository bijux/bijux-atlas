---
title: Automation Command Surface
audience: maintainer
type: reference
status: canonical
owner: atlas-docs
last_reviewed: 2026-07-22
---

# Automation Command Surface

The maintainer namespace is `bijux dev atlas ...`; the repository-local binary
is `bijux-atlas-dev`. Put `--repo-root <path>` before the command family when
invoking a binary outside the repository root.

## Discover Before Executing

```bash
bijux dev atlas list --format json
bijux dev atlas describe <runnable-id> --format json
bijux dev atlas check list --format json
bijux dev atlas suites list --format json
bijux dev atlas reports list --format json
```

Discovery output is build-specific. The governed command registry defines the
public top-level families; the check, suite, runnable, and report registries
define the objects those families discover.

## Public Families

| Family | Responsibility |
| --- | --- |
| `check` / `checks` | list, explain, run, and diagnose registered checks. |
| `suites` | list, describe, run, inspect, compare, and lint executable suite registries. |
| `reports` | list report contracts, render their index, show registry gaps, and validate report directories. |
| `docs` | validate, inspect, generate, build, serve, and smoke-test the documentation site. |
| `governance` | inspect rules, exceptions, deprecations, compatibility, doctrine, and ADR state. |
| `configs` / `policies` / `registry` | validate and explain governed repository inputs. |
| `ci` | inspect and verify CI lane definitions and environment contracts. |
| `tests` / `perf` / `load` | select test, performance, and load-validation workflows. |
| `ops` / `system` / `observe` | inspect deployment, diagnostics, observability, and operational contracts. |
| `security` / `audit` | validate security policy, auth, threats, dependencies, and audit surfaces. |
| `api` / `runtime` | validate API and runtime contracts. |
| `datasets` / `ingest` / `tutorials` | validate data and worked-evidence workflows. |
| `list` / `describe` / `run` / `validate` | operate on registry-backed runnables and validation profiles. |

The exact governed list is stored in
`configs/sources/governance/governance/cli-dev-command-surface.json`. Hidden
commands and aliases remain implementation support unless that registry and a
maintainer contract expose them.

## Current Registry Drift

The checked-in dev command registry and the current Clap command tree do not
fully agree. The registry declares `clients`, `contract`, `demo`, and `packages`
as top-level families, but the CLI does not expose those top-level variants.
The CLI exposes `migrations`, while the registry omits it.

Until that drift is resolved, use binary help to determine what can execute and
the registry to identify the intended governed surface. Do not add automation
that depends on any mismatched family. Nested commands such as `api contract`
remain separate from a top-level `contract` family.

## Global Controls

| Option | Meaning |
| --- | --- |
| `--repo-root <path>` | select the repository whose registries and files are evaluated. |
| `--output-format human\|json\|both` | override supported local renderers globally. |
| `--json` | request JSON through the legacy global switch where supported. |
| `--quiet` | reduce human-facing output. |
| `--verbose` / `--debug` | increase diagnostic detail. |
| `--fail-fast` | stop eligible orchestration after the first blocking failure. |
| `--print-policies` | include policy selection details in supported execution output. |
| `--print-boundaries` | include execution-boundary details in supported output. |
| `--no-deprecation-warn` | suppress deprecation warnings without changing behavior. |

Local `--format` flags are command-specific. The common choices are `text` or
`json`; suite rendering uses `human`, `json`, or `both`. Read the selected
subcommand's help instead of assuming one global format vocabulary.

## Check Selection

`check list` and `check run` accept selectors for `--suite`, `--domain`,
`--severity`, `--mode`, `--tag`, `--name`, and `--id`. Slow and internal checks
remain excluded unless requested.

```bash
bijux dev atlas check list --domain docs --format json
bijux dev atlas check explain checks_docs_index_links --format json
bijux dev atlas check run \
  --id checks_docs_index_links \
  --format json
```

`--mode static` selects read-only checks. `--mode effect` selects checks that
may need granted capabilities. `--include-slow` changes selection; it is not a
capability grant.

## Suite Execution

The executable suite registry currently exposes `checks`, `contracts`, and
`tests`.

```bash
bijux dev atlas suites describe --suite contracts --format json
bijux dev atlas suites run \
  --suite contracts \
  --mode pure \
  --format json
```

Suite modes are `pure`, `effect`, and `all`. Runs accept an explicit artifact
root and run ID, plus group or tag filters. Use `suites last`, `suites report`,
`suites history`, and `suites diff` to inspect retained runs rather than
reconstructing outcomes from terminal output.

## Capability-Gated Commands

Commands that can spawn subprocesses, write files, inspect Git, or use the
network expose matching flags:

```text
--allow-subprocess  --allow-write  --allow-git  --allow-network
```

Grant only the effects required by the selected command. A refusal caused by a
missing capability is evidence that work did not run, not a passing result.

## Compatibility Boundary

A maintainer may depend on a documented family, selector, exit behavior, and a
field governed by its exact output schema. Human wording, hidden aliases,
internal modules, command ordering in help, and unregistered report shapes are
not compatibility promises.

When a public family changes, the Clap implementation, dev command registry,
affected output schema, and consuming wrappers must agree. Generated help and
indexes are then refreshed as observations of that coordinated change.

See [Automation Control Plane](automation-control-plane.md) for authority and
capability flow, and [Automation Reports Reference](automation-reports-reference.md)
for report discovery and validation.
