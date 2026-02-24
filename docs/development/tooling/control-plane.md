# Control Plane: bijux-dev-atlas

`bijux dev atlas` is the canonical development control-plane for this repository.

Use it (directly or through thin Makefile wrappers) for:

- checks and suites (`check ...`, `gates ...`)
- docs validation/build (`docs ...`)
- configs validation/inventory (`configs ...`)
- ops validation/render/status (`ops ...`)
- control-plane policies (`policies ...`)

Runtime command surface remains separate:

- runtime CLI: `bijux atlas ...`
- control plane: `bijux dev atlas ...`

Legacy `atlasctl` pages are historical references only and must not be used for new workflows.
