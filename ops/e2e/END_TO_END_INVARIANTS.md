# End-to-end Invariants

- E2E suites must produce deterministic pass/fail status for identical fixture inputs.
- E2E summary output must include run id, suite id, and reproducible status fields.
- Scenario fixtures must map to declared suites and evidence outputs without orphan mappings.
- Generated e2e evidence must validate against committed schemas before merge.
- E2E regressions must block release readiness when invariant status is non-pass.
