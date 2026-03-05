# Clients

Client SDK products live under `crates/`.

## Canonical client locations

- `crates/bijux-atlas-client-python/`

## Python client layout

- `python/`: package source (`atlas_client`)
- `examples/`: runnable usage examples
- `tests/`: product tests
- `docs/`: client documentation
- `notebooks/`: notebook assets

## Policy

- Root `clients/` is forbidden.
- Repository automation for clients must run through `bijux-dev-atlas clients ...`.
- Python and notebook files are only allowed in approved crate zones defined in `configs/governance/allowed-nonrust.json`.
