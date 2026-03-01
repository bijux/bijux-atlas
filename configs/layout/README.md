# Layout configs

- Owner: `platform`
- Purpose: set tree and budget expectations for repository structure.
- Consumers: repository layout contracts and budget enforcement.
- Update workflow: change budgets only with an approved structural change, then rerun repository and config layout contracts.

Canonical layout policy:
- `configs/<domain>/<group>/<file>`
- Maximum path depth: `4`
- Maximum group depth from domain: `2`
