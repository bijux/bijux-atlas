# Make Contract
#
# Scope
# - Make public targets are thin delegates to `bijux dev atlas ...` or `cargo ...`.
# - Operational behavior SSOT is the Rust control plane, not Makefiles.
#
# Public target policy
# - Public targets must be listed in `make/help.md`.
# - Public targets must have one-line descriptions.
# - Public targets must write outputs under `artifacts/`.
# - Public targets must not hide network usage; networked targets must be explicit.
#
# Internal target policy
# - Internal targets use `_internal-` prefix.
# - Internal targets do not appear in public help output.
#
# Shell policy
# - No `cd` chains in recipes.
# - No multiline shell pipelines in public targets.
# - No script execution from `scripts/` in public targets.
#
# Ownership
# - CI and reviewers use `make` only as an entrypoint wrapper.
# - `bijux dev atlas` remains the behavior source of truth.
