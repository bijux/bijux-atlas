# Production Readiness Checklist

## Scope
This checklist is the final go/no-go gate before a `bijux-atlas` production rollout.
Use `docs/product/REFERENCE_GRADE_ACCEPTANCE_CHECKLIST.md` during PR review before this final gate.

## SLO and Incident Readiness
- [ ] SLO targets are documented in `docs/observability/SLO.md`.
- [ ] Canonical SLO target file is current (`docs/product/SLO_TARGETS.md`).
- [ ] Error budget policy is approved and active (`docs/observability/error-budget-policy.md`).
- [ ] Alert rules are deployed (`docs/observability/alert-rules.yaml`).
- [ ] Runbooks are current:
  - [ ] `docs/runbooks/STORE_OUTAGE.md`
  - [ ] `docs/runbooks/DATASET_CORRUPTION.md`
  - [ ] `docs/runbooks/HIGH_MEMORY.md`

## Safety Controls
- [ ] Rate limits and concurrency caps are configured for expected traffic.
- [ ] Cache limits are set (`max_disk_bytes`, `max_dataset_count`, TTL).
- [ ] Circuit breaker thresholds are configured for store instability.
- [ ] `cached_only_mode` profile is tested for outage degradation.

## Data Integrity
- [ ] Artifact checksum verification passes for all pinned datasets.
- [ ] Periodic re-verification schedule is configured.
- [ ] Catalog refresh and epoch reporting are validated.
- [ ] Dataset pinning set is reviewed and intentional.

## Deployment
- [ ] Container image is reproducible and tagged immutably.
- [ ] Helm values for resource requests/limits are tuned.
- [ ] Readiness/liveness/startup probes are validated in staging.
- [ ] Rollback playbook is validated (`docs/runbooks/ROLLBACK_PLAYBOOK.md`).

## Validation Before Promotion
- [ ] `make dev-test-all` passes.
- [ ] `make dev-audit` passes with no warnings/errors.
- [ ] Load scenario passes baseline (`docs/load/k6.md`).
- [ ] K8s chaos/perf scenarios executed (`docs/runbooks/K8S_PERF_CHAOS.md`).
- [ ] Cold-start and warm-cache latency are within target.
