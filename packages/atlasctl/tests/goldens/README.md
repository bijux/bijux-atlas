# Atlasctl Goldens

All golden snapshots under this directory are SSOT test fixtures for atlasctl CLI behavior.

## Update policy

- Do not edit golden files manually.
- Regenerate goldens only via:
  - `./bin/atlasctl gen goldens`
  - or `atlasctl gen goldens`
- Review the diff and commit only intentional behavior changes.

## Layout

- `check/`: check-run and check-list golden outputs.
- `list/`: list/commands inventory goldens.
- `help/`: help surface and help text snapshots.
- `contracts/`: schema/output contract representative payloads.
- `suite/`: suite inventories and suite run snapshot text.
- `samples/`: sample schema payloads.

`MANIFEST.json` is generated and must match files on disk.
