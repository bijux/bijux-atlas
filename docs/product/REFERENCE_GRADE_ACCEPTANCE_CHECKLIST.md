# Reference-Grade Acceptance Checklist

Use this checklist in PR review for production-grade changes.

1. Contract stability: API/artifact changes are additive or versioned.
2. Determinism: no nondeterministic ordering/time-dependent outputs introduced.
3. Boundaries: effects remain in allowed crates/modules only.
4. Policy compliance: limits/strictness enforced and tested.
5. Guardrails: lint/tests/docs gates remain green.
6. Error quality: stable machine codes and actionable details maintained.
7. Performance: query plan/index behavior verified where relevant.
8. Observability: metrics/logs/traces updated for new paths.
9. Operations: runbooks/docs updated for new failure modes.
10. Security/supply chain: audit posture unchanged or improved.
11. Upgrade path: compatibility and rollback implications documented.
12. Test quality: positive, negative, and regression tests included.
