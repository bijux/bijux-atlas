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

- `all`: checks=160 markers=['configs', 'dev', 'docs', 'ops', 'policies']
- `bypass-governance`: checks=71 markers=['check', 'policies']
- `checks-all`: checks=0 markers=['check']
- `checks-fast`: checks=510 markers=['check', 'fast']
- `ci`: checks=512 markers=['required']
- `ci-nightly`: checks=2 markers=['slow']
- `ci-pr`: checks=510 markers=['fast']
- `configs`: checks=2 markers=['configs']
- `control-plane-gates`: checks=13 markers=['check', 'ci']
- `dev`: checks=0 markers=['dev']
- `docs`: checks=20 markers=['docs']
- `drill-nightly`: checks=84 markers=['ops', 'slow']
- `internal`: checks=0 markers=['internal', 'internal-only']
- `k8s-e2e-nightly`: checks=84 markers=['kube', 'ops', 'slow']
- `lint-all`: checks=140 markers=['lint']
- `lint-fast`: checks=510 markers=['fast', 'lint']
- `load-baseline`: checks=84 markers=['load', 'ops', 'slow']
- `load-smoke`: checks=510 markers=['fast', 'load', 'ops']
- `local`: checks=510 markers=['fast']
- `obs-verify`: checks=84 markers=['obs', 'ops', 'slow']
- `ops`: checks=82 markers=['ops']
- `ops-deploy`: checks=82 markers=['deploy', 'ops']
- `ops-load`: checks=82 markers=['load', 'ops']
- `ops-nightly`: checks=84 markers=['ops', 'slow']
- `ops-obs`: checks=82 markers=['obs', 'ops']
- `ops-run-guardrails`: checks=152 markers=['lint', 'ops']
- `ops-stack`: checks=82 markers=['ops', 'stack']
- `perf-nightly`: checks=84 markers=['ops', 'slow']
- `policies`: checks=65 markers=['policies']
- `product`: checks=0 markers=['product']
- `product.smoke`: checks=0 markers=['product']
- `refgrade-audit`: checks=143 markers=['ops', 'policies', 'slow']
- `repo-hygiene`: checks=517 markers=['fast', 'hygiene', 'repo']
- `required`: checks=512 markers=['required']
- `required_proof`: checks=516 markers=['required']
- `slow`: checks=2 markers=['slow']
- `stack-nightly`: checks=84 markers=['ops', 'slow']
