---
title: Monitoring quickstart
owner: bijux-atlas-operations
stability: stable
last_reviewed: 2026-03-05
---

# Monitoring quickstart

1. Load alert rules from:
   - `ops/observe/alerts/atlas-alert-rules.yaml`
   - `ops/observe/alerts/slo-burn-rules.yaml`
2. Import dashboards from:
   - `ops/observe/dashboards/atlas-observability-dashboard.json`
   - `ops/observe/dashboards/atlas-slo-dashboard.json`
   - `ops/observe/dashboards/atlas-runtime-health-dashboard.json`
   - `ops/observe/dashboards/atlas-query-performance-dashboard.json`
   - `ops/observe/dashboards/atlas-ingest-pipeline-dashboard.json`
   - `ops/observe/dashboards/atlas-artifact-registry-dashboard.json`
   - `ops/observe/dashboards/atlas-system-resources-dashboard.json`
3. Verify contracts:
   - `bijux dev atlas ops observe slo verify --allow-write --format json`
   - `bijux dev atlas ops observe alerts verify --allow-write --format json`
   - `bijux dev atlas ops observe runbooks verify --allow-write --format json`
4. Generate readiness report:
   - `bijux dev atlas ops readiness --allow-write --format json`
