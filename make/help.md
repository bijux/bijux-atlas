# Make Public Targets

The public surface is defined by `CURATED_TARGETS` in `makefiles/root.mk` and printed by `make help`.

- Public targets must have one-line descriptions.
- Internal targets must use `_internal-` prefix.
- Public targets delegate to `bijux dev atlas ...` or `cargo ...`.
- Public targets write output under `artifacts/`.
