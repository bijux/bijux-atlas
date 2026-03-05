# Reproducibility Limitations

Current limits:

1. Container image digest equality may vary by builder/runtime implementation.
2. Release manifest artifact hash coverage depends on produced local artifacts.
3. Docs reproducibility relies on stable input files and excludes generated runtime caches.
