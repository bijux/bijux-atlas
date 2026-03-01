# Docs Index

- [Architecture](architecture.md)
- [Plugin Contract](PLUGIN_CONTRACT.md)
- [Effects](effects.md)
- [Config Discovery](CONFIG_DISCOVERY.md)
- [CLI Command List](CLI_COMMAND_LIST.md)
- [CLI UX Contract](CLI_UX_CONTRACT.md)
- [Exit Codes](EXIT_CODES.md)
- Ingest contract reference: [`../../bijux-atlas-ingest/docs/ingest-contract.md`](../../bijux-atlas-ingest/docs/ingest-contract.md)
- [Public API](public-api.md)
- [Tests](../tests/)
- [Benches](../benches/)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `docs/public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/public-api.md`, and add targeted tests.

- Central docs index: docs/index.md
- Crate README: ../README.md
