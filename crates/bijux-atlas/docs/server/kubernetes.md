# Kubernetes Ops

- Configure cache root on writable volume (`emptyDir` or persistent volume).
- Use read-only root filesystem with dedicated writable cache mount.
- Readiness requires catalog reachability unless `cached-only` mode is enabled.
- Use pinned datasets and startup warmup for hot-path datasets.
- Use startup warmup jitter (`ATLAS_STARTUP_WARMUP_JITTER_MAX_MS`) to reduce startup stampede.
- Configure graceful drain (`ATLAS_SHUTDOWN_DRAIN_MS`) so pods stop taking new traffic before exit.
- Prefer cached-only mode for degraded operation during store outages.
- See canonical chart `ops/k8s/charts/bijux-atlas/` and runbooks under root docs for rollout/rollback.
