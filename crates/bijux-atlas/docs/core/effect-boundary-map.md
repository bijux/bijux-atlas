# Effect Boundary Map

This crate follows the Atlas effect-boundary policy from `docs/effects-boundary-maps.md`.

- Keep pure domain/planning logic in pure modules.
- Route any filesystem/network/time/random effects through explicit adapter modules.
- Do not leak effect adapters into public API surfaces.
