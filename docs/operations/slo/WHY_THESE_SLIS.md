# Why These SLIs

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

These SLIs are the minimal set that covers user pain and platform truth.

## User pain coverage

- Can I reach the service now: availability (`/readyz`) and success rate by class.
- Is it fast enough: latency by endpoint class.
- Does degraded mode still protect core paths: cheap survival and shed rate.
- Are datasets usable: cache ratio, dataset availability, store dependency behavior.

## Platform truth coverage

- Registry and dataset freshness are represented explicitly.
- Correctness guard captures silent semantic regressions across equivalent APIs.
- Cold-start captures startup readiness impact seen by first real queries.

## Why not more SLIs

- Extra SLIs without ownership or actionability create alert fatigue.
- Each included SLI has a direct runbook or remediation path.
