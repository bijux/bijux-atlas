# CI Overview

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: describe human-level CI guarantees and lane behavior.

## CI Guarantees

- Required contracts are enforced before merge.
- Docs and config surfaces are validated deterministically.
- Release readiness is gated by explicit validation lanes.

## Lanes

- Pull request validation
- Documentation validation
- Operations validation
- Nightly validation
- Release candidate validation

### Required Contracts Lane Map

- `local`: contributor preflight checks.
- `pr`: required contracts and policy gates on pull requests.
- `merge`: required contracts with merge-lane evidence requirements.
- `release`: required contracts with release-readiness validation.

## Reports in CI

See [CI report consumption](../control-plane/ci-report-consumption.md) for artifact flow and ownership.

## Verify Success

A CI run is healthy when required lanes pass and evidence artifacts are present and parseable.

## What to Read Next

- [Control-plane](../control-plane/index.md)
- [Release Workflow](../operations/release-workflow.md)
- [Contributing](contributing.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
