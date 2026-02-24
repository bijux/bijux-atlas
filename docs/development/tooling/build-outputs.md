# Build Outputs

`bijux dev atlas build ...` owns build artifacts for the Rust control plane.

Artifacts live under `artifacts/`:

- `artifacts/bin/`: runnable binaries and `manifest.json`
- `artifacts/build/cargo/`: isolated cargo target directories for build lanes
- `artifacts/dist/`: release bundles (`.tar.gz`) and `sha256sum.txt`
- `artifacts/reports/`: structured reports

Use thin Make wrappers:

- `make build`
- `make dist`
- `make clean-build`
- `make build-doctor`

The Makefiles delegate. `bijux dev atlas build ...` is the SSOT for naming, paths, and bundle layout.
