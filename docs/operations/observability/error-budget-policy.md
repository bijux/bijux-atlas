# Bijux Atlas Error Budget Policy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

## Budget Definition

- Availability SLO baseline: 99.5% success over rolling 30 days.
- Error budget: 0.5% failed requests over rolling 30 days.

## Burn Thresholds

- Fast burn: >10% of monthly budget consumed in 24h.
- Sustained burn: >2% of monthly budget consumed per day for 3 consecutive days.

## Required Actions

- Fast burn:
  - freeze risky deploys
  - enable tighter concurrency/rate limits
  - prioritize incident mitigation over feature rollout
- Sustained burn:
  - initiate reliability sprint
  - defer non-reliability roadmap work until burn normalizes

## Exit Criteria

- Burn rate below sustained threshold for 7 days.
- No active Sev-1/Sev-2 incidents tied to request reliability.

## See also

- `ops-ci`
