---
title: Operations
audience: operator
type: concept
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-01
tags:
  - operations
  - runtime
related:
  - docs/reference/index.md
  - docs/architecture/index.md
---

# Operations

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: provide the canonical operator entrypoint across run, deploy, observe, and incident workflows.

## Purpose

Give operators one stable entrypoint for deploy, observe, rollback, and incident handling.

## Entry Points

1. [Operations overview](overview.md)
2. [Operator quickstart](operator-quickstart.md)
3. [Run locally](run-locally.md)
4. [Deploy](deploy.md)
5. [Installing ops product](deploy/installing-ops-product.md)
5. [Minimal production overrides](minimal-production-overrides.md)
6. [Install verification checklist](install-verification-checklist.md)
7. [Observability setup](observability-setup.md)
8. [Validation entrypoints](validation-entrypoints.md)
9. [Kind simulation](kind-simulation.md)
10. [Upgrade validation](upgrade.md)
11. [Chart semver policy](chart-semver-policy.md)
12. [Profile upgrade policy](profile-upgrade-policy.md)
13. [Rollback limitations](rollback-limitations.md)
14. [Incident response](incident-response.md)
15. [Release](ops/release/index.md)
16. [Release evidence](release-evidence.md)
17. [Release candidate checklist](release-candidate-checklist.md)
18. [Asset index](../_assets/index.md)
18. [Evidence viewer](evidence-viewer.md)
19. [Security posture](security-posture.md)
20. [Docs site deployment](docs-site-deploy.md)
21. [Institutional drills](drills.md)
22. [Institutional readiness checklist](institutional-readiness-checklist.md)
23. [Review packet](review-packet.md)
23. [Performance operations](performance.md)
24. [Query performance benchmarks](query-performance-benchmarks.md)
25. [Query benchmark summary report](query-benchmark-summary-report.md)
26. [Performance regression policy](performance-regression-policy.md)
27. [Performance regression monitoring](performance-regression-monitoring.md)
28. [Performance CI operations](performance-ci-operations.md)
29. [Performance philosophy](performance-philosophy.md)
30. [Performance metrics reference](performance-metrics-reference.md)
31. [Performance regression guide](performance-regression-guide.md)
32. [Performance dashboard guide](performance-dashboard-guide.md)
33. [Performance troubleshooting guide](performance-troubleshooting-guide.md)
34. [Performance architecture diagrams](performance-architecture-diagrams.md)
35. [Performance testing tutorial](performance-testing-tutorial.md)
36. [Performance roadmap](performance-roadmap.md)
37. [Performance FAQ](performance-faq.md)
38. [Performance glossary](performance-glossary.md)
39. [Performance design principles](performance-design-principles.md)
40. [Performance optimization guidelines](performance-optimization-guidelines.md)
41. [Dataset update](dataset-update.md)
42. [Dataset deprecation](dataset-deprecation.md)
43. [Data retention policy](data-retention-policy.md)
44. [Data access model](data-access-model.md)
45. [Scientific defensibility](scientific-defensibility.md)
46. [Cluster deployment models](cluster-deployment-models.md)
47. [Cluster upgrade and compatibility](cluster-upgrade-and-compatibility.md)
48. [Cluster membership protocol](cluster-membership-protocol.md)
49. [Cluster node lifecycle states](cluster-node-lifecycle-states.md)
50. [Cluster node health monitoring](cluster-node-health-monitoring.md)
51. [Cluster node maintenance procedures](cluster-node-maintenance-procedures.md)
52. [Cluster troubleshooting guide](cluster-troubleshooting-guide.md)
53. [Shard distribution guide](shard-distribution-guide.md)
54. [Shard rebalance workflow](shard-rebalance-workflow.md)
55. [Shard failure scenarios](shard-failure-scenarios.md)
56. [Shard debugging guide](shard-debugging-guide.md)
57. [Replication failover process](replication/failover-process.md)
58. [Replication troubleshooting](replication/troubleshooting.md)
59. [Replication metrics dashboards](replication/metrics-dashboards.md)
60. [Replication policy](replication/policy.md)
61. [Replication scaling strategy](replication/scaling-strategy.md)
62. [Resilience recovery workflow](resilience/recovery-workflow.md)
63. [Resilience failure injection guide](resilience/failure-injection-guide.md)
64. [Resilience failure scenarios](resilience/failure-scenarios.md)
65. [Resilience recovery workflows](resilience/recovery-workflows.md)
66. [Resilience runbooks](resilience/runbooks.md)
67. [Disaster recovery plan](resilience/disaster-recovery-plan.md)
68. [Resilience recovery timeline examples](resilience/recovery-timeline-examples.md)
69. [Resilience dashboard examples](resilience/dashboard-examples.md)
70. [Resilience completion report](resilience/completion-report.md)
71. [Ops OCI chart consumer workflow](ops-oci-chart-consumer-workflow.md)
72. [Ops offline bundle consumer workflow](ops-offline-bundle-consumer-workflow.md)
73. [Ops chart upgrade guide](ops-chart-upgrade-guide.md)
74. [Ops chart rollback guide](ops-chart-rollback-guide.md)
75. [Ops profile selection guide](ops-profile-selection-guide.md)
76. [Ops provenance](ops-provenance.md)
77. [Ops compatibility matrix](ops-compatibility-matrix.md)

78. [Ops as product](ops-as-product.md)
79. [Verify ops artifacts authenticity](verify-ops-artifacts-authenticity.md)
80. [Release artifacts overview](release-artifacts-overview.md)
81. [Verify release artifacts](verify-release-artifacts.md)
82. [Reproducible build posture](reproducible-build-posture.md)
83. [Publishing channels](publishing-channels.md)
84. [Release lifecycle and support window](release-lifecycle-support-window.md)
85. [Governance enforcement](governance-enforcement.md)
86. [Governance rule reference](governance-rule-reference.md)
87. [Governance enforcement workflow](governance-enforcement-workflow.md)
88. [Governance enforcement troubleshooting](governance-enforcement-troubleshooting.md)
89. [Governance evolution policy](governance-evolution-policy.md)
90. [Governance rule schema](governance-rule-schema.md)
91. [Governance enforcement completion report](governance-enforcement-completion-report.md)
92. [Audit report schema](audit-report-schema.md)
93. [Audit troubleshooting guide](audit-troubleshooting-guide.md)
94. [Audit runbook](audit-runbook.md)
95. [System audit completion report](system-audit-completion-report.md)

## What This Page Is Not

This page is not a command reference and not an architecture deep dive.
Operational policies are enforced by contracts such as `OPS-ROOT-023` and `OPS-ROOT-017`.
The docs surface stays in `docs/operations/**`; contract sources stay in `docs/_internal/contracts/**`.

## Verify Success

Operator workflows are successful when each linked page reaches a concrete verification outcome.

## Next steps

Use [Reference](../reference/index.md) for exact flags and schemas, and [Runbooks](runbooks/index.md) during incidents.
Also review the glossary for canonical terms.
For product intent and boundaries, read [What is Bijux Atlas](../product/what-is-bijux-atlas.md).

## Document Taxonomy

- Audience: `operator`
- Type: `guide`
- Stability: `stable`
- Owner: `bijux-atlas-operations`
