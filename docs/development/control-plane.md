# Control Plane

Owner: `platform`  
Type: `guide`  
Audience: `contributor`  
Reason to exist: define stable `bijux dev atlas` command families and ownership boundaries.

## Command Families

- `check`
- `docs`
- `configs`
- `ops`
- `policies`

## Purpose

The control plane enforces deterministic governance checks and operational orchestration without ad-hoc scripts.

## Exit Codes

- `0`: all selected checks passed
- `1`: one or more non-required checks failed
- `2`: usage error
- `3`: internal runner error
- `4`: one or more required checks failed

## CI Mode

- Use `bijux dev atlas ... --ci` for CI-facing runs.
- CI mode forces CI profile behavior and disables ANSI color output.

## Operations Boundary

- `ops/` stores operational data, schemas, and runbooks.
- User-facing operational execution uses `bijux dev atlas ops ...` (or thin `make` wrappers), not raw script entrypoints.
