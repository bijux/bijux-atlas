---
title: Automation Control Plane
audience: maintainer
type: guide
status: canonical
owner: atlas-docs
last_reviewed: 2026-03-15
---

# Automation Control Plane

Atlas uses `bijux dev atlas ...` as the canonical installed automation surface for repository checks, docs workflows, governance validation, and machine-readable evidence. The direct binary that backs that namespace is `bijux-dev-atlas`.

## Why This Exists

```mermaid
flowchart LR
    Scripts[Ad hoc scripts] --> Drift[Behavior drift]
    Drift --> HiddenRules[Hidden policy]
    HiddenRules --> FragileCI[Fragile CI]
    ControlPlane[bijux dev atlas] --> SharedRules[Shared command surface]
    SharedRules --> Evidence[Deterministic evidence]
    Evidence --> Reviewable[Reviewable automation]
```

The goal is simple: one execution surface, one capability model, and one place to document automation behavior.

## Surface Model

```mermaid
flowchart TD
    Make[make wrappers] --> DevAtlas[bijux dev atlas]
    DevAtlas --> Suites[suites]
    DevAtlas --> Checks[check]
    DevAtlas --> Docs[docs]
    DevAtlas --> Governance[governance]
    DevAtlas --> Reports[reports]
    Suites --> Artifacts[artifacts and reports]
    Checks --> Artifacts
    Docs --> Artifacts
    Governance --> Artifacts
    Reports --> Artifacts
```

Use `make` for the common lane wrappers and `bijux dev atlas ...` when you need narrower selection or deeper inspection.

## Lane and Selection Rules

The broad workflow is:

- `make ci-fast` for fast local feedback
- `make ci-pr` for the pull-request lane
- `make ci-nightly` for broader and slower coverage
- `make docs-build` for docs-specific build validation

The narrow workflow is:

```bash
bijux dev atlas suites list
bijux dev atlas check list
cargo run -q -p bijux-dev-atlas -- suites list
cargo run -q -p bijux-dev-atlas -- check run --suite ci_pr --include-internal --include-slow --allow-git --format json
cargo run -q -p bijux-dev-atlas -- check list
cargo run -q -p bijux-dev-atlas -- check run --tag lint --format json
```

Pick the smallest surface that matches the question you are answering. Do not bypass required lanes by inventing a different command path.

## Static and Effect Boundaries

Some workflows are pure reads, while others intentionally require effects.

- `check run` declares `static` versus `effect` execution modes
- `suites run` can be constrained with `--mode pure`, `--mode effect`, or `--mode all`
- docs commands that spawn tools or write artifacts require explicit capability flags such as `--allow-subprocess`, `--allow-write`, and `--allow-network`

Commands should fail closed when a required capability is missing. Quietly downgrading behavior would make CI and local evidence diverge.

## Triage Workflow

When automation fails:

1. Re-run the matching wrapper or suite first.
2. Prefer JSON output when the lane consumes structured reports.
3. Inspect the named check, suite, or report before changing code.
4. Apply the smallest fix that restores the documented contract.
5. Re-run the focused command and then the broader lane.

Common entry points:

```bash
make ci-pr
make docs-build
cargo run -q -p bijux-dev-atlas -- governance check --format json
cargo run -q -p bijux-dev-atlas -- reports index --format json
```

## Operational Guardrails

- repository automation should be routed through `bijux dev atlas ...`, not ad hoc root scripts
- expensive or environment-sensitive validations belong in the correct lane, not hidden inside fast feedback loops
- external tools and capability requirements should fail with remediation, not with mystery
- evidence should describe the failure, the rerun command, and the relevant artifact path

## Where to Go Next

- [Contributor Workflow](contributor-workflow.md)
- [Testing and Evidence](testing-and-evidence.md)
- [Command Surface](../07-reference/command-surface.md)
