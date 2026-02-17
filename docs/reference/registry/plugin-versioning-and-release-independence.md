# Plugin Versioning And Release Independence

## Independent Versioning

- Umbrella (`bijux`) and plugins (`bijux-*`) version independently.
- Compatibility is enforced via plugin metadata field `compatible_umbrella`.
- Umbrella must refuse incompatible plugins at dispatch time.

## Release Cadence Independence

- Plugins may ship patch/minor releases without umbrella release.
- Umbrella may release without forcing plugin rebuilds.
- Only compatibility-range changes require coordinated announcement.

## What

Reference definition for this topic.

## Why

Defines stable semantics and operational expectations.

## Scope

Applies to the documented subsystem behavior only.

## Non-goals

Does not define unrelated implementation details.

## Contracts

Normative behavior and limits are listed here.

## Failure modes

Known failure classes and rejection behavior.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
