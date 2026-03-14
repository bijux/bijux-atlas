# Benchmark Coverage Targets

This page captures the long-lived benchmark coverage expected from `bijux-dev-atlas`.

## Scenario coverage

- Query benchmarks should cover point lookup, region lookup, and multi-result scans.
- Ingest benchmarks should cover parsing, normalization, and artifact materialization paths.
- Dataset tiers should cover at least small, medium, and large fixture classes.

## Evidence lifecycle

- Benchmark history should be retained in deterministic JSON artifacts.
- Benchmark summaries should remain publishable into generated docs artifacts.
- Baseline refreshes should stay reviewable and traceable through normal repository evidence.

## Regression analysis

- Performance diffs should support threshold-based regression classification.
- Reproducibility checks should compare repeated runs under fixed isolation inputs.
- Coverage should include dataset-tier matrix validation for major benchmark families.

## Execution environments

- Local deterministic execution remains the default benchmark surface.
- Cluster-oriented benchmark profiles should stay explicit and reproducible.
- Sustained load and distributed execution should be modeled as governed extensions, not ad hoc scripts.
