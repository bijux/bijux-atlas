# Production Readiness Checklist

## Scope
This checklist is the final go/no-go gate before a `bijux-atlas` production rollout.
Use `docs/product/reference-grade-checklist.md` during PR review before this final gate.

## SLO and Incident Readiness
- [ ] SLO targets are documented in `docs/observability/slo.md`.
- [ ] Canonical SLO target file is current (`docs/product/slo-targets.md`).
- [ ] Error budget policy is approved and active (`docs/observability/error-budget-policy.md`).
- [ ] Alert rules are deployed (`docs/observability/alert-rules.yaml`).
- [ ] Runbooks are current:
  - [ ] `docs/runbooks/store-outage.md`
  - [ ] `docs/runbooks/dataset-corruption.md`
  - [ ] `docs/runbooks/high-memory.md`

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
- [ ] Rollback playbook is validated (`docs/runbooks/rollback-playbook.md`).

## Validation Before Promotion
- [ ] `make dev-test-all` passes.
- [ ] `make dev-audit` passes with no warnings/errors.
- [ ] Load scenario passes baseline (`docs/load/k6.md`).
- [ ] K8s chaos/perf scenarios executed (`docs/runbooks/k8s-perf-chaos.md`).
- [ ] Cold-start and warm-cache latency are within target.
