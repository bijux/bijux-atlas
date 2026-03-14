# Benchmark Architecture

```mermaid
flowchart LR
    A[perf command] --> B[scenario loader]
    B --> C[benchmark runner]
    C --> D[result validator]
    D --> E[JSON and CSV artifacts]
    E --> F[history and diff analysis]
```

## Components

- Configuration and isolation model: `crates/bijux-dev-atlas/src/performance/config.rs`
- Dataset registry and tiers: `crates/bijux-dev-atlas/src/performance/dataset.rs`
- Result and diff model: `crates/bijux-dev-atlas/src/performance/harness.rs`
- Runtime command surface: `crates/bijux-dev-atlas/src/commands/perf.rs`
