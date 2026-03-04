# Refresh Pinned GitHub Actions SHAs

- Owner: `bijux-atlas-security`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the canonical way to refresh GitHub Actions SHA pins without weakening the supply chain boundary.

## Preconditions

- Review the current action inventory in `ops/inventory/toolchain.json`.
- Confirm the target upstream action tag or release note before changing any pinned SHA.

## Safe Refresh Flow

1. Update the pinned `uses:` SHA in the workflow file and the matching `github_actions` entry in
   `ops/inventory/toolchain.json`.
2. Run `bijux-dev-atlas security validate --format json`.
3. Review `artifacts/security/security-github-actions.json` to confirm every workflow reference is SHA pinned and
   inventory-aligned.
4. If a temporary non-SHA exception is unavoidable, record it in
   `configs/security/github-actions-exceptions.json` with owner, reason, and expiry.

## Prohibited Shortcuts

- Do not point workflows at mutable tags without a governed exception.
- Do not update workflow SHAs without updating the matching inventory entry.
- Do not leave an expired exception in place after the pin can be restored.
