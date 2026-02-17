# Bijux Security Response Coordination Policy

- Security reports must use private disclosure channels.
- Cross-project incidents require a named incident coordinator.
- Coordinated release process:
  1. validate impact across umbrella/plugins
  2. prepare patched releases per affected project
  3. publish advisory with fixed versions and mitigations
- Backports are required for still-supported release lines.

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
