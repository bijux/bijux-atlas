# Build Outputs

`bijux dev atlas build ...` owns build artifacts for the Rust control plane.

Artifacts live under `artifacts/`:

- `artifacts/dist/bin/`: runnable binaries and `manifest.json`
- `artifacts/build/cargo/`: isolated cargo target directories for build lanes
- `artifacts/dist/build.json`: build metadata (no wall-clock timestamp by default)
- `artifacts/dist/release/`: release bundles (`.tar.gz`) and `sha256sum.txt`
- `artifacts/reports/`: structured reports

Use thin Make wrappers:

- `make build`
- `make build-release`
- `make build-ci`
- `make build-meta`
- `make dist`
- `make dist-verify`
- `make clean-dist`
- `make clean-build`
- `make build-doctor`

`make build-sdist` is intentionally forbidden. Source tarball packaging is not part of the Rust
control-plane contract.

## Local PATH Strategy

For local testing of built binaries, prepend `artifacts/dist/bin` to `PATH` explicitly when needed.
Do not rely on root `bin/` shims.

The Makefiles delegate. `bijux dev atlas build ...` is the SSOT for naming, paths, and bundle layout.
