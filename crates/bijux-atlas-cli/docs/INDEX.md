# Docs Index

- [Architecture](ARCHITECTURE.md)
- [Plugin Contract](PLUGIN_CONTRACT.md)
- [Effects](EFFECTS.md)
- [Config Discovery](CONFIG_DISCOVERY.md)
- [CLI Command List](CLI_COMMAND_LIST.md)
- [Public API](PUBLIC_API.md)
- [Tests](../tests/)
- Benches: none

## API stability

Public API is defined only by `docs/PUBLIC_API.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/PUBLIC_API.md`, and add targeted tests.

