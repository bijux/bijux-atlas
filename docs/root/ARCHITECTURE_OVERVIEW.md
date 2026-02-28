# Architecture Overview

- Owner: `docs-governance`
- Stability: `stable`

`bijux-atlas` is organized as domain crates, runtime crates, and control-plane crates.

- Domain foundation: `bijux-atlas-core`, `bijux-atlas-model`
- Data and query path: `bijux-atlas-ingest`, `bijux-atlas-store`, `bijux-atlas-query`
- Product interfaces: `bijux-atlas-api`, `bijux-atlas-server`, `bijux-atlas-cli`
- Development control plane: `bijux-dev-atlas*`

Detailed architecture references:
- `docs/architecture/INDEX.md`
- `docs/development/crates-map.md`
