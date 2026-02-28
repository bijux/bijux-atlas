# Architecture

Owner: `architecture`  
Type: `concept`  
Audience: `contributor`  
Reason to exist: define canonical system structure and enduring boundaries.

## System Shape

`bijux-atlas` is organized around domain crates, runtime crates, and control-plane crates.

- Domain foundation: `bijux-atlas-core`, `bijux-atlas-model`
- Data and query path: `bijux-atlas-ingest`, `bijux-atlas-store`, `bijux-atlas-query`
- Product interfaces: `bijux-atlas-api`, `bijux-atlas-server`, `bijux-atlas-cli`
- Development control plane: `bijux-dev-atlas`

## Canonical Pages

- [Boundaries](boundaries.md)
- [Architecture Map](architecture-map.md)
