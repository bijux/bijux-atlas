# Fasta-Derived Metrics Contract

- Owner: `bijux-atlas-ingest`
- Stability: `stable`

## Scope

This contract defines optional metrics derived by scanning FASTA during ingest.

## Contract

- Feature flag: `fasta_scanning_enabled` (default `false`).
- When disabled, ingest uses `.fai` for contig lengths only and does not scan FASTA content.
- When enabled with `compute_contig_fractions=true`, contig GC/N fractions are computed deterministically and written to SQLite `contigs` table.
- Scan order is deterministic (file order, streaming parser).
- Memory guardrail: `fasta_scan_max_bases` hard-limits total scanned bases; exceeding it fails ingest.

## Serve Implication

v1 serving does not require FASTA content at runtime; `.fai` and derived SQLite are sufficient unless optional FASTA-derived metrics are requested at ingest time.
