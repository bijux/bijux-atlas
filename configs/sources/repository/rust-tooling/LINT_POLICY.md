# Rust Lint Policy

Rust lint behavior is defined by a small set of stable inputs:
- `rustfmt.toml` controls formatting.
- `clippy.toml` controls lint configuration.
- `toolchain.json` pins the expected Rust toolchain contract for repository automation.

Policy:
- Prefer fixing code to adding new lint exceptions.
- Land lint policy changes with the code or workflow change that requires them.
- Treat formatting and lint config path changes as CI contract changes, because workflows and local tooling reference these files directly.

This directory should stay limited to Rust toolchain and lint inputs. Broader repository policy belongs elsewhere under `configs/`.
