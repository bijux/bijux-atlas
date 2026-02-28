# Known limitations

- Owner: `platform`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@331751e4`
- Reason to exist: document current control-plane limitations without hiding merge-gate requirements.

## Current limitations

- Some heavy validations are lane-restricted for local ergonomics.
- Some environment-sensitive checks are CI-only by design.
- Some legacy contracts remain temporarily ignored until rewrite completion.
- Some older docs and generated registries still need sentence-case and canonical-link cleanup.
- Some reports exist only as JSON contracts today and still need generated reader reference pages.

## Verify success

This page remains short, specific, and synchronized with actual gate behavior.

## Next steps

- [Debug failing checks](debug-failing-checks.md)
- [CI report consumption](ci-report-consumption.md)
- [Tooling dependencies](tooling-dependencies.md)
