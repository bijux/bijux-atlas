# Artifact Directory Contract (SSOT)

- Owner: `docs-governance`

Canonical layout:

`release=<release>/species=<species>/assembly=<assembly>/`

Inside dataset directory:

- `inputs/genes.gff3.bgz`
- `inputs/genome.fa.bgz`
- `inputs/genome.fa.bgz.fai`
- `derived/gene_summary.sqlite`
- `derived/manifest.json`
- `derived/release_gene_index.json`
- Optional diff artifacts for cross-release comparisons:
  - `derived/diffs/from=<from_release>/to=<to_release>/diff.json`
  - `derived/diffs/from=<from_release>/to=<to_release>/diff.summary.json`
  - `derived/diffs/from=<from_release>/to=<to_release>/chunks/*.json`

This layout is encoded in `bijux-atlas-model::artifact_paths`.

## What

Defines a stable contract surface for this topic.

## Why

Prevents ambiguity and drift across CLI, API, and operations.

## Scope

Applies to atlas contract consumers and producers.

## Non-goals

Does not define internal implementation details beyond the contract surface.

## Contracts

Use the rules in this page as the normative contract.

## Failure modes

Invalid contract input is rejected with stable machine-readable errors.

## Examples

```bash
$ make ssot-check
```

Expected output: a zero exit code and "contract artifacts generated" for successful checks.

## How to verify

Run `make docs docs-freeze ssot-check` and confirm all commands exit with status 0.

## See also

- [Contracts Overview](../INDEX.md)
- [SSOT Workflow](../ssot-workflow.md)
- [Terms Glossary](../../_style/terms-glossary.md)
