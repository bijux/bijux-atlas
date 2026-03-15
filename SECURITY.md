# Security Policy

Security reports for Bijux Atlas should stay private until maintainers have reproduced the issue, scoped the impact, and prepared a fix or mitigation.

## Supported Versions

Atlas currently supports:

- `main`
- the latest `0.1.x` release line

Older versions are unsupported unless the repository explicitly documents an extension in release material or contracts.

## How To Report A Vulnerability

Use a GitHub private security advisory for this repository. Do not open a public issue for an unpatched vulnerability.

Include:

- affected component or path
- affected versions or commit range
- impact and expected attacker capability
- reproduction steps or proof of concept
- known mitigations or likely fix direction
- logs, traces, or artifacts only if they are safe to share privately

The repository already includes a security advisory intake template at [`.github/ISSUE_TEMPLATE/security-advisory.yml`](.github/ISSUE_TEMPLATE/security-advisory.yml).

## What Counts As Security-Relevant Here

Security work in Atlas is not limited to one crate. Treat these areas as security-sensitive:

- authentication, authorization, and request-boundary behavior in runtime surfaces
- data exposure through APIs, logs, traces, or generated artifacts
- release provenance, signing, supply-chain, and dependency integrity
- container, CI, and workflow behavior that could weaken trust in shipped artifacts
- docs, configs, or ops inputs that define or explain the live security posture

If a change can alter how operators secure Atlas or how users trust its outputs, it is security-relevant.

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

This repository does not promise a bug bounty, private consulting, or support for fork-specific modifications. The supported security surface is the one described by the current repository, the numbered docs spine, and the active release line.
