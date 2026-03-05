# Why Scripts Are Forbidden

Repository-local automation scripts (`tools/`, `scripts/`, root `*.sh`, root `*.py`) are forbidden to keep operational behavior stable and auditable.

## Institutional reasons

1. Single execution surface: automation must be discoverable through `bijux-dev-atlas` commands.
2. Uniform policy enforcement: command execution goes through one policy and capability model.
3. Deterministic evidence: report and artifact shapes stay stable across CI and local runs.
4. Reviewability: governance checks can validate one automation boundary instead of many ad hoc paths.
5. Long-term maintainability: command names and behavior are versioned and documented in one place.
