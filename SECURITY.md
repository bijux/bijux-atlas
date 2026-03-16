# Security Policy

Last updated: 2026-03-16

Security reports for Bijux Atlas should stay private until maintainers have reproduced the issue, scoped the impact, and prepared a fix or mitigation.

## Supported Versions

Security fixes are applied to the latest released Atlas runtime line.
For this policy, "released" means an official tagged release with published artifacts from this repository.
The `main` branch and workspace-only maintainer tooling are reviewed on a best-effort basis while fixes are being prepared, but older released lines are unsupported unless the release material says otherwise.

## How To Report A Vulnerability

Preferred:

- GitHub private report: <https://github.com/bijux/bijux-atlas/security/advisories/new>

Do not open a public issue for an unpatched vulnerability.

Include:

- affected component or path
- affected install surface or command route (`bijux-atlas`, `bijux-atlas-server`, `bijux-atlas-openapi`, `bijux atlas ...`, or `bijux dev atlas ...`)
- affected versions or commit range
- impact and expected attacker capability
- reproduction steps or proof of concept
- known mitigations or likely fix direction
- logs, traces, or artifacts only if they are safe to share privately

The repository already includes a security advisory intake template at [`.github/ISSUE_TEMPLATE/security-advisory.yml`](.github/ISSUE_TEMPLATE/security-advisory.yml).

If the report depends on the `bijux atlas ...` umbrella route, say whether the defect is in Atlas itself or in the sibling `bijux-cli` host/runtime layer.

## What Counts As Security-Relevant Here

Security work in Atlas is not limited to one crate. Treat these areas as security-sensitive:

- authentication, authorization, and request-boundary behavior in runtime surfaces
- data exposure through APIs, logs, traces, or generated artifacts
- release provenance, signing, supply-chain, and dependency integrity
- container, CI, and workflow behavior that could weaken trust in shipped artifacts
- docs, configs, or ops inputs that define or explain the live security posture

If a change can alter how operators secure Atlas or how users trust its outputs, it is security-relevant.

## Response Expectations

This project is maintained on a best-effort basis.

Current targets:

- acknowledgement within 3 business days
- triage or next-step update within 7 business days

Complex issues can take longer to fix, especially when they touch release, docs, or repository-governance surfaces together.

## Fix Expectations

Security fixes should be handled with the same honesty as code fixes:

- reproduce the issue first
- minimize blast radius while fixing it
- update tests, docs, configs, ops inputs, and contracts when they are part of the security story
- record compatibility or operational fallout explicitly
- ship the advisory or release note only after the fix story is clear

Urgency can shorten the path, but it does not remove the need for evidence.

## Security References

- operational guidance: [`docs/04-operations/security-operations.md`](docs/04-operations/security-operations.md)
- release model: [`docs/06-development/release-and-versioning.md`](docs/06-development/release-and-versioning.md)
- operational promises: [`docs/08-contracts/operational-contracts.md`](docs/08-contracts/operational-contracts.md)

## Boundaries

This repository does not promise a bug bounty, private consulting, or support for fork-specific modifications.
The supported security surface is the one described by the current repository, the numbered docs spine, and the latest published Atlas release line.
