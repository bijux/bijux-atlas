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

- `all`: checks=36 markers=['docs', 'dev', 'ops', 'policies', 'configs']
- `ci`: checks=249 markers=['required']
- `ci-nightly`: checks=3 markers=['slow']
- `ci-pr`: checks=246 markers=['fast']
- `configs`: checks=2 markers=['configs']
- `dev`: checks=0 markers=['dev']
- `docs`: checks=18 markers=['docs']
- `drill-nightly`: checks=7 markers=['ops', 'slow']
- `internal`: checks=0 markers=['internal', 'internal-only']
- `k8s-e2e-nightly`: checks=7 markers=['ops', 'slow', 'kube']
- `local`: checks=246 markers=['fast']
- `ops`: checks=4 markers=['ops']
- `ops-nightly`: checks=7 markers=['ops', 'slow']
- `perf-nightly`: checks=7 markers=['ops', 'slow']
- `policies`: checks=12 markers=['policies']
- `required`: checks=249 markers=['required']
- `required_proof`: checks=249 markers=['required']
- `slow`: checks=3 markers=['slow']
- `stack-nightly`: checks=7 markers=['ops', 'slow']
