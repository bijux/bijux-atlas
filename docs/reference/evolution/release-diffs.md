# Release Diffs

- Owner: `bijux-atlas-cli` + `bijux-atlas-server`
- Stability: `evolving`

## What

Cross-release diffs are first-class artifacts generated from two published dataset artifacts.

Diff identity is:
- `from_release`
- `to_release`
- `species`
- `assembly`

## Why

Release diffs provide deterministic, auditable evolution signals without requiring runtime API computation.

## Scope

Covers artifact generation (`atlas diff build`), schema (`docs/contracts/DIFF_SCHEMA.json`), and local ops smoke validation.

## Non-Goals

- No dedicated persisted diff API endpoint contract in this phase.
- No transcript-level diff semantics beyond optional placeholders.

## Contracts

- `atlas diff build` writes both `diff.json` and `diff.summary.json`.
- Diff schema is versioned (`schema_version = "1"`).
- Compatibility promise: diff schema changes are additive-only.
- Stable identity strategy:
  - prefer stable gene ID (`gene_id`)
  - fallback key: `seqid:start-end` if `gene_id` is absent
- Size guardrail:
  - large lists are chunked under `chunks/*.json`
  - summary remains bounded and always emitted.

## Failure Modes

- Missing release gene index or sqlite biotype table causes build failure.
- Invalid dataset identity dimensions fail fast.
- Chunk output write failures fail the command.

## How To Verify

```bash
$ cargo run -p bijux-atlas-cli -- atlas diff build \
  --root artifacts/store \
  --from-release 110 \
  --to-release 111 \
  --species homo_sapiens \
  --assembly GRCh38 \
  --out-dir artifacts/diff-smoke
$ make ops-diff-smoke
```

Expected output: `diff.json` and `diff.summary.json` created with deterministic hash on repeated runs.

## See Also

- [Contracts Index](../contracts/contracts-index.md)
- [Artifact Directory Contract](../contracts/artifacts/directory-contract.md)
- [Product Diffs v1](../product/diffs-v1.md)
- [Terms Glossary](../_style/terms-glossary.md)
