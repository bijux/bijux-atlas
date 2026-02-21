# DEC-0001 Control Plane SSOT

## Decision

`atlasctl` is the control-plane SSOT; Makefiles are dispatch-only wrappers.

## Consequence

- Business logic and policy enforcement live in `atlasctl` commands/checks.
- Make targets delegate to `atlasctl` and avoid direct script/python logic.
