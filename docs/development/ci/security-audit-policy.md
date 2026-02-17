# Security Audit Policy

- `cargo deny` and `cargo audit` run on every CI cycle.
- Baseline allowlist is permitted only for explicitly tracked advisories with rationale.
- Embargoed advisories:
  - Do not disclose details publicly before embargo lifts.
  - Track internally with owner + remediation timeline.
  - Remove temporary exceptions immediately after fix release.
