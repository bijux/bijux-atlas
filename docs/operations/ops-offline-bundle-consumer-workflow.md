# Ops Offline Bundle Consumer Workflow

Use the offline bundle when cluster environments cannot access registries.

1. Build or download `ops-bundle-vX.Y.Z.tar.gz`.
2. Verify checksums from `checksums.json`.
3. Extract and install from local chart package:
```bash
tar -xzf ops-bundle-v0.1.0.tar.gz -C /tmp/atlas-ops
helm install atlas /tmp/atlas-ops/ops/k8s/charts/bijux-atlas -f /tmp/atlas-ops/ops/k8s/values/offline.yaml
```
4. Run bundle verification:
```bash
bijux dev atlas release ops bundle-verify --version 0.1.0 --format json
```
