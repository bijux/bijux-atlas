# Kubernetes Ops

- Configure cache root on writable volume (`emptyDir` or persistent volume).
- Use read-only root filesystem with dedicated writable cache mount.
- Readiness should require catalog reachability when enabled.
- Use pinned datasets and startup warmup for hot-path datasets.
- Prefer cached-only mode for degraded operation during store outages.
- See root `charts/bijux-atlas/` and runbooks under root docs for rollout/rollback.
