# Ops Taxonomy

Canonical ops areas for atlasctl:

- `stack`: local/shared infra stack lifecycle and stack contracts
- `deploy`: deploy/apply/rollback orchestration for atlas runtime surfaces
- `k8s`: kubernetes render, validation, conformance, and k8s-oriented checks
- `obs`: observability packs, drills, validation, and conformance
- `load`: load generation, baseline comparison, and perf report flows
- `e2e`: end-to-end scenarios, smoke, soak, and real-data drills
- `datasets`: dataset fetch/publish/qc/warmup/fixtures orchestration
- `pins`: tool/version pin policy and validation
- `reports`: unified report generation and report post-processing

Notes:
- `ops` itself is the command group, not a leaf area.
- Internal migration helpers are not taxonomy areas and must live under `commands/ops/internal/**`.
