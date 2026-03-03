# Institutional Reader

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `guide`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@a10951a3e4e65b3b9be3bb67b16b4dc16a6d5287`
- Last changed: `2026-03-03`
- Reason to exist: point external reviewers to the available evidence surfaces.

## Available Evidence

- Profile intent and ownership: [Operations profiles](operations/profiles.md)
- Runtime and command contracts: [Reference contracts](reference/contracts/index.md)
- Generated documentation diagnostics: [Docs health dashboard](_internal/generated/docs-health-dashboard.md)
- Supply-chain and release controls: [Operations supply-chain policies](operations/supply-chain-policies.md)
- Exception posture and inventory: [Governance exceptions](operations/governance-exceptions.md)
- Compatibility posture: [_internal governance compatibility process](_internal/governance/compatibility-and-deprecation-process.md)

## What This Page Is For

Use this page when you need a review packet starting point for traceability, contract coverage, and
operator posture.

## How To Read Compatibility Artifacts

- `artifacts/governance/deprecations-summary.json` lists the governed overlap windows.
- `artifacts/governance/breaking-changes.json` lists active governed breaking changes.
- `artifacts/governance/governance-doctor.json` is the quick deterministic summary for reviewers.
- `artifacts/governance/institutional-delta.md` is the human-readable release delta.
