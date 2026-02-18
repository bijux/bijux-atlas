# Local Stack (Make Only)

Run the local reference stack using make targets only.

```bash
make ops-up
make ops-deploy
make ops-publish
make ops-warm
make ops-smoke
make ops-k8s-tests
make ops-load-smoke
make ops-observability-validate
```

One-command flow:

```bash
make ops-full
```

Canonical targets: `ops-up`, `ops-deploy`, `ops-warm`, `ops-smoke`, `ops-k8s-tests`, `ops-load-smoke`, `ops-observability-validate`, `ops-full`.
