# Coverage Runner Notes

Coverage uses `cargo-llvm-cov` on Linux CI runners.

Runner differences:

- Linux CI is the coverage source of truth.
- macOS local runs may produce different absolute paths and symbolization details.
- Use CI artifacts for gate decisions.
