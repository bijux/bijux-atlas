# Suites

Generated from suite registries (`pyproject.toml` + `src/atlasctl/registry/suites_catalog.json`).

- Default suite: `required`

## Configured Suites

- `ci`: includes=['required'] items=2 complete=False
- `fast`: includes=[] items=4 complete=False
- `ops`: includes=[] items=2 complete=False
- `release_0_1`: includes=['required_proof'] items=3 complete=True
- `required`: includes=['fast'] items=7 complete=True
- `required_proof`: includes=['ci'] items=7 complete=True

## First-Class Suites

- `all`: checks=159 markers=['docs', 'dev', 'ops', 'policies', 'configs']
- `ci`: checks=409 markers=['required']
- `ci-nightly`: checks=3 markers=['slow']
- `ci-pr`: checks=406 markers=['fast']
- `configs`: checks=2 markers=['configs']
- `dev`: checks=0 markers=['dev']
- `docs`: checks=20 markers=['docs']
- `drill-nightly`: checks=85 markers=['ops', 'slow']
- `internal`: checks=0 markers=['internal', 'internal-only']
- `k8s-e2e-nightly`: checks=85 markers=['ops', 'slow', 'kube']
- `load-baseline`: checks=85 markers=['ops', 'load', 'slow']
- `load-smoke`: checks=406 markers=['ops', 'load', 'fast']
- `local`: checks=406 markers=['fast']
- `obs-verify`: checks=85 markers=['ops', 'obs', 'slow']
- `ops`: checks=82 markers=['ops']
- `ops-deploy`: checks=82 markers=['ops', 'deploy']
- `ops-load`: checks=82 markers=['ops', 'load']
- `ops-nightly`: checks=85 markers=['ops', 'slow']
- `ops-obs`: checks=82 markers=['ops', 'obs']
- `ops-stack`: checks=82 markers=['ops', 'stack']
- `perf-nightly`: checks=85 markers=['ops', 'slow']
- `policies`: checks=64 markers=['policies']
- `product`: checks=0 markers=['product']
- `product.smoke`: checks=0 markers=['product']
- `refgrade-audit`: checks=143 markers=['ops', 'policies', 'slow']
- `required`: checks=409 markers=['required']
- `required_proof`: checks=409 markers=['required']
- `slow`: checks=3 markers=['slow']
- `stack-nightly`: checks=85 markers=['ops', 'slow']
