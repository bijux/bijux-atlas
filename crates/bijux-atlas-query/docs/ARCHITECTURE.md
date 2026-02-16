# Architecture

## Architecture

Modules:
- `filters`: request/response/filter types + projection compiler.
- `cursor`: cursor payload encoding/decoding + signature checks.
- `planner`: classification, limits validation, work estimation.
- `sql`: SQL generation, normalization hash, explain/index checks.
- `limits`: policy-driven query limit settings.
