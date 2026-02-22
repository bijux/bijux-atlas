# Bypass Policy: Temporary by Default

Bypasses, relaxations, and allowlists exist to keep delivery moving while a real fix is being implemented.

Default rule:
- removal is the default
- retention is an exception
- every new bypass needs ownership, justification, and an expiry

## Principles

- Prefer narrowing scope over broad wildcards.
- Prefer command/check fixes over allowlist growth.
- Prefer short expiries and milestone-linked removal plans.
- Treat approvals as temporary migration tools, not permanent policy.

## Required Metadata

Every bypass entry should carry:
- `owner`
- `issue_id` (or documented local policy reference where allowed)
- `expiry`
- `reason` / justification
- `removal_plan`

## Zero-Bypass Program

The long-term target is zero bypasses for stable lanes. Burn-down is enforced through:
- inventory reporting
- trend gates
- explicit new-entry approvals
- expiry validation
- removal milestone tracking
