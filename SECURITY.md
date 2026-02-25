# Security Policy

## Supported Version
- `v0.1.x` (current release line)

## Reporting a Vulnerability
- Open a private security advisory in GitHub for this repository.
- Include affected crate(s), impact, reproduction steps, and proposed mitigation.

## Security Baseline
- Rust-only control plane and runtime governance commands.
- Pinned dependency and toolchain policy enforced in CI.
- No direct script/tool bypasses outside approved command surfaces.
- Deterministic artifacts with explicit contracts.

## Triage and Fix
- Reproducible proof and scope assessment are required.
- Security fixes are prioritized, reviewed by code owners, and released with changelog notes.
