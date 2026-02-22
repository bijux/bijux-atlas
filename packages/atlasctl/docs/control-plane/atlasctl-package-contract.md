# Atlasctl Contract

## Purpose
`atlasctl` is the single source of truth (SSOT) control plane for repository automation.
Makefiles are wrappers only.

## Core Rules
1. SSOT ownership:
`atlasctl` owns orchestration, policy checks, execution flow, and reporting behavior.

2. Wrapper purity:
Makefiles must delegate to `./bin/atlasctl ...` only.
Makefiles must not run toolchains directly (`cargo`, `pytest`, `kubectl`, `helm`, etc.).

3. Canonical gate verbs:
`fmt`, `lint`, `test`, `audit`, `check`, `build`, `clean`, `gen`, `doc`, `ops`.

4. Gate variants:
`X` is default/fast path.
`X-all` is full/slow path (includes ignored tests or extended checks).

5. Rare variant:
`X-and-checks` may exist only when required by a documented operational reason.

6. Canonical domain list (max 10):
`dev`, `ci`, `docs`, `ops`, `configs`, `policies`, `packages`, `registry`, `reporting`, `internal`.

7. Public vs internal domains:
Public CLI groups: `dev`, `ci`, `docs`, `ops`, `configs`, `policies`, `packages`, `registry`, `reporting`.
Internal-only: `internal`.

8. Target naming uniqueness:
No duplicate names that map to the same behavior.
Example forbidden pattern: both `fmt` and `dev-fmt` as duplicate public surfaces.

9. Ownership metadata:
Every gate must have an owner in SSOT metadata.
Ownership must be declared in metadata files, not comments.

10. Artifact output:
Every gate writes under `artifacts/reports/...`.
Every gate must respect isolation env vars (for example `ISO_ROOT`, `ISO_TAG`, `ISO_RUN_ID`).

11. Forbidden wording policy:
Do not use blocked wording listed in
`configs/policy/forbidden-adjectives.json` unless explicitly approved in
`configs/policy/forbidden-adjectives-approvals.json`.

## Enforcement
- Contract checks must fail CI on wrapper impurity or ownership coverage gaps.
- Contract checks must fail CI on blocked wording usage outside explicit approvals.
