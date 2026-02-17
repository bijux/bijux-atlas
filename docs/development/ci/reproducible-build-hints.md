# Reproducible Build Hints

- Commit and enforce `Cargo.lock` in CI (`--locked` for release builds).
- Use pinned Rust toolchain via `rust-toolchain.toml`.
- Keep target dirs isolated per CI job under `artifacts/isolates/<job>/`.
- Avoid environment-dependent build scripts and timestamps in artifacts.
