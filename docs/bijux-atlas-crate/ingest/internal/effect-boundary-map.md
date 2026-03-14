# Effect Boundary Map

This domain follows the Atlas effect-boundary policy documented in `../../internal/effect-boundary-map.md`.

- Keep pure domain/planning logic in pure modules.
- Route any filesystem/network/time/random effects through explicit adapter modules.
- Do not leak effect adapters into public API surfaces.
