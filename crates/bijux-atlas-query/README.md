# bijux-atlas-query

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Deterministic query parsing, planning, and execution for atlas gene/transcript read paths.

## Intended Use

- parse request payloads into typed AST
- produce pure typed plans with explicit cost hooks
- execute plans against SQLite-backed read models

## Supported Query Subset

- exact `gene_id`
- exact `name`
- `name_prefix`
- exact `biotype`
- region overlap (`seqid/start/end`)
- transcript filters (`parent_gene_id`, `biotype`, `transcript_type`, region)

## Determinism Guarantees

- stable AST normalization and formatting
- stable planner outputs for identical semantic requests
- stable ordering in query results and explain-plan normalization

## Docs

- `docs/query-language-spec.md`
- `docs/ordering.md`
- `docs/pagination.md`
- `docs/cost-estimator.md`

## Purpose
- Describe the crate responsibility and stable boundaries.

## How to use
- Read `docs/index.md` for workflows and examples.
- Use the crate through its documented public API only.

## Where docs live
- Crate docs index: `docs/index.md`
- Contract: `CONTRACT.md`
