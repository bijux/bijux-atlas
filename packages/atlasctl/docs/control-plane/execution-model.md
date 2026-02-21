# Control Plane Execution Model

## Principle

Checks and command planners are pure; effectful operations are centralized behind core boundaries.

## Effect Boundaries

- Filesystem writes: `atlasctl.core.fs`
- Subprocess execution: `atlasctl.core.exec`
- Environment variable reads: `atlasctl.core.env`
- Process lifecycle/timeouts: `atlasctl.core.process`
- Network I/O: `atlasctl.core.network`

Any exception must be explicit, reviewed, and temporary.

## Nesting Policy

Target policy from `packages/atlasctl/src/atlasctl`:

- Maximum package depth: `4`
- New work should prefer depth `<=3`

## Makefile Boundary

Make targets dispatch to `atlasctl` commands and do not implement business logic directly.
