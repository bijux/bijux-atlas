# bijux-atlas-policies

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Policy contracts, validation rules, and machine-readable enforcement results for atlas runtime behavior.

## Scope

This crate owns:
- `PolicySet` parsing and validation
- policy evaluation output (`PolicyViolation`)
- severity taxonomy (`PolicySeverity`)
- repository budget evaluation (`RepositoryMetrics`)

## Extend Policies

1. Add data fields in `configs/policy/policy.json` and schema updates in `configs/policy/policy.schema.json`.
2. Add or update evaluation rules with stable IDs.
3. Add table-driven tests and golden fixtures.

## Interpret Violations

Every violation includes:
- `id`: stable machine identifier
- `severity`: `info`, `warning`, or `error`
- `message`: deterministic rule summary
- `evidence`: concrete field/value context

## Docs

- `docs/policy-authoring-guide.md`
- `docs/schema.md`
- `docs/config-schema.md`
- `docs/public-api.md`

## Purpose
- Describe the crate responsibility and stable boundaries.

## How to use
- Read `docs/index.md` for workflows and examples.
- Use the crate through its documented public API only.

## Where docs live
- Crate docs index: `docs/index.md`
- Contract: `CONTRACT.md`
