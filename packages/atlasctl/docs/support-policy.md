# Support Policy

This document defines support expectations for the `atlasctl` surface.

## Stable vs Internal

- Stable commands are documented under `docs/commands/` and appear in public help.
- Internal commands are hidden by default and may change without compatibility guarantees.
- Internal commands are only shown when `--include-internal` is requested.

## Stability Guarantees

- Stable command names and required flags are preserved within the same major line.
- Stable JSON outputs follow schema versioning and no-breaking-change checks.
- Internal-only outputs are not contract-stable.

## Deprecation

- A stable command/flag slated for removal must be documented in release notes first.
- Removal must land with migration guidance in docs.
