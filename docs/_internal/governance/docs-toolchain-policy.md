# Docs Toolchain Policy

- Owner: `docs-governance`
- Review cadence: `quarterly`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define how the documentation toolchain is updated and reproduced.

## Update Cadence

- Review `configs/docs` dependencies weekly through Dependabot.
- Accept patch and minor updates when lint and docs build outputs stay compatible.
- Investigate major updates deliberately before merging.

## Breaking Lints

- If a tool upgrade introduces stricter lints, either fix the docs to meet the new rule or pin the
  previous known-good version until the rule can be adopted cleanly.
- Do not float tool versions to bypass a breaking lint change.

## Pinning Rule

- `configs/docs/package.json` must pin npm tooling versions exactly.
- `configs/docs/requirements.txt` and `configs/docs/requirements.lock.txt` must pin Python tooling
  versions exactly.
- Local reproduction must use the committed lockfiles instead of ad-hoc latest installs.

## SBOM Scope

The docs toolchain SBOM surface includes the committed npm and Python lockfiles plus the packages
they pin. It does not include transient local editor plugins or user-specific global tools.

## Reproduce Locally

1. Run `npm ci --prefix configs/docs`.
2. Install Python packages from `configs/docs/requirements.lock.txt`.
3. Run the documented docs validation and build commands.
