# Attack Surface

## Entrypoints

- HTTP endpoints: `/healthz`, `/readyz`, API handlers, and metrics
- Environment inputs: runtime config env vars and secret env values
- Helm and values inputs: chart values, profile overlays, and network policy mode controls
- CI and control-plane execution: release evidence, simulation, validation, and drill commands
- Release artifacts: manifests, tarballs, SBOMs, and summary reports
