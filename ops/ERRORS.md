# Ops Errors

Mapping of ops-facing failure categories to bijux-dev-atlas error reporting.

| Category | Typical bijux-dev-atlas command family | Error shape | Notes |
|---|---|---|---|
| Contract validation | `bijux dev atlas ops * validate/check` | non-zero + schema/contract message | Prefer deterministic schema path in output |
| Prerequisite missing | `ops prereqs`, `ops kind`, `ops stack` | prereq failure / missing tool | Include missing binary/version |
| Drift detected | `ops gen check`, surface/schema checks | drift message + remediation command | Must print canonical regen command |
| Runtime orchestration failure | `ops up/down/deploy/e2e/load/obs` | subprocess/step failure | Include failed step id and log path |
| Policy denial | lint/policy checks | explicit policy violation | Include owning policy/check id |
| Migration guard | `ops migrate`, legacy path checks | cutoff or duplicate-path error | Include cutoff date + canonical path |
