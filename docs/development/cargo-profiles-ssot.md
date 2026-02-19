# Cargo Profiles SSOT

- Owner: `rust-platform`

## What

Single source of truth for nextest profile names used by make targets.

## Profiles

- `ci`: default deterministic profile for CI-safe lanes.
- `fast-unit`: fast unit-focused subset for tight local loops.
- `slow-integration`: slower integration coverage profile.
- `certification`: certification profile for release-quality suites.

## Where Set

- Default profile variables are defined in `makefiles/cargo.mk`:
  - `NEXTEST_PROFILE`
  - `NEXTEST_PROFILE_FAST`
  - `NEXTEST_PROFILE_SLOW`
  - `NEXTEST_PROFILE_CERT`
- Profile definitions and filters live in `configs/nextest/nextest.toml`.
- Public entrypoints that choose profiles:
  - `make cargo/test-fast` sets `NEXTEST_PROFILE=fast-unit`
  - `make cargo/test` sets `NEXTEST_PROFILE=ci`
  - `make cargo/test-all` sets `NEXTEST_PROFILE=ci`
  - `make cargo/coverage` sets `NEXTEST_PROFILE=ci`

## Contract

- CI-safe targets must use explicit nextest profiles.
- `make root` and `lane-cargo` must not depend on `cargo-dev.mk` targets.
- Cargo invocations outside `makefiles/cargo.mk` and `makefiles/cargo-dev.mk` are forbidden.
