# Stack Dependencies

Required tools for local ops stack:

- kind
- kubectl
- helm
- curl

Optional tools:

- k6 (load/perf scenarios)
- kubeconform (manifest conformance in `ops-stack-validate`)

Required Kubernetes capabilities:

- DNS (CoreDNS)
- default StorageClass
- ability to create namespaces, deployments, services, configmaps, secrets, jobs
