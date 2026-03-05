# Ops OCI Chart Consumer Workflow

Use the published OCI Helm chart as the primary online distribution surface.

1. Authenticate to `ghcr.io`.
2. Pull chart metadata:
```bash
helm show chart oci://ghcr.io/bijux/charts/bijux-atlas
```
3. Install with explicit profile values:
```bash
helm install atlas oci://ghcr.io/bijux/charts/bijux-atlas \
  --version 0.1.0 \
  -f ops/k8s/values/prod.yaml
```
4. Verify rollout and evidence outputs with `bijux dev atlas release ops validate-package`.
