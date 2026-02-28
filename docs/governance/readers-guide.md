# Governance Readers Guide

- Owner: `docs-governance`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Reason to exist: explain how the reader-facing docs system is organized and maintained.

## How Atlas Docs Are Organized

- Reader-facing sections: `product`, `architecture`, `api`, `operations`, `development`, `reference`
- Contributor governance: `governance`
- Tooling outputs: `_generated`
- Quarantine area: `_drafts`

## Why This Split Exists

Readers need stable explanations and procedures.
Contributors need policy and review controls.
Generated artifacts support validation but are not part of reader navigation.

## Reader Spine Maintenance Rules

- Update spine links in `docs/index.md` first.
- Keep section index links curated and below 10.
- Preserve one golden path per audience.

## Next

- [Docs Charter](docs-charter.md)
- [Governance Index](index.md)
