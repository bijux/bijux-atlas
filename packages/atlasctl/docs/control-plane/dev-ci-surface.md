# DEV/CI Surface

This document is the SSOT for the supported DEV/CI control-plane entrypoints.

## Stable Front Door

- CI one-liner: `atlasctl dev ci run`
- Developer shortcuts:
- `atlasctl dev fmt`
- `atlasctl dev lint`
- `atlasctl dev check`
- `atlasctl dev test`
- `atlasctl dev test --all`
- `atlasctl dev test --contracts`
- `atlasctl dev coverage`
- `atlasctl dev audit`

## Stability Policy

- `atlasctl dev ci run` is the stable public CI entrypoint.
- `atlasctl suite run ci` is internal plumbing and not the stable public CI front door.
- Makefiles must call stable `atlasctl` entrypoints only.
- Makefiles must not call the internal suite engine directly.

## Human vs CI Use

- Humans should run the stable front door commands above.
- CI workflows should use the same stable front door (`atlasctl dev ci run`) for CI suite execution.
