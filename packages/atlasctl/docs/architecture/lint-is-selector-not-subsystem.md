# Lint Is A Selector, Not A Subsystem

`lint` is an execution category and selection surface, not a separate execution engine.

## What Runs Lint

- The canonical runner in `atlasctl.execution.runner`
- `atlasctl check run` (function checks + categorized rows)
- `atlasctl suite run ...` (first-class suites)
- `atlasctl lint <suite>` (command-check suites routed through the same runner payload model)

## Why

- One payload contract: `atlasctl.check-run.v1`
- One progress/events stream model
- One attachments model
- One timing/budget behavior contract

## Practical Rule

When adding lint coverage, add a selector/category/marker (for example `lint`,
`fast`, `network`, `write`) and route it through the canonical runner.
Do not create a new subsystem-specific runner.
