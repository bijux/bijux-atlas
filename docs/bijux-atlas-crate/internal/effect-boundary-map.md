# Effect Boundary Map

This page defines the Atlas effect-boundary policy for the merged `bijux-atlas` crate.

- Keep pure domain/planning logic in pure modules.
- Route any filesystem/network/time/random effects through explicit adapter modules.
- Do not leak effect adapters into public API surfaces.
