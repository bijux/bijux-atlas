# Governance Suites

## Purpose

This directory is the authority surface for governed validation suite definitions.
Each suite file declares a stable set of executable entries and the metadata needed
to route them through the control plane.

## Files

- `checks.suite.json`: quality-gate checks run through the suite runner.
- `contracts.suite.json`: governance invariants run through the suite runner.
- `tests.suite.json`: reserved suite surface for explicitly governed test entrypoints.
- `suites.index.json`: inventory of all governed suite files in this directory.

## Ownership

The `team:atlas-governance` owner maintains this directory. Changes here must stay
aligned with the registries in `configs/governance/` and with the suite validation
rules in `crates/bijux-dev-atlas`.
