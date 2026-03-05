---
title: Tutorials Doctrine
audience: contributor
type: policy
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - tutorials
  - automation
  - governance
---

# Tutorials doctrine

Tutorial assets are evidence-bearing artifacts. Tutorial automation is owned by `bijux-dev-atlas`.

## Rules

- Tutorial workflows must run through `bijux-dev-atlas tutorials ...` commands.
- Tutorial outputs must be deterministic and reproducible for the same inputs.
- Tutorial generated outputs must be committed as generated artifacts, not hand-edited.
- `tutorials/` is for artifacts and tutorial-facing assets; repository automation scripts are not allowed there after migration closure.
- Legacy tutorial automation (`tutorials/scripts/*` and `tutorials/tests/*.py`) is retired; validation and workflow execution live under `crates/bijux-dev-atlas`.

## Directory purpose

- `tutorials/`: dataset contracts, evidence, dashboards, and tutorial-facing static assets.
- `docs/tutorials/`: tutorial narrative and generated references.

## Required replacement command surface

- `bijux-dev-atlas tutorials list`
- `bijux-dev-atlas tutorials explain`
- `bijux-dev-atlas tutorials verify`
- `bijux-dev-atlas tutorials run workflow`
- `bijux-dev-atlas tutorials build docs`
- `bijux-dev-atlas tutorials dataset package`
- `bijux-dev-atlas tutorials dataset ingest`
- `bijux-dev-atlas tutorials dataset integrity-check`
- `bijux-dev-atlas tutorials reproducibility-check`
- `bijux-dev-atlas tutorials workspace cleanup`
