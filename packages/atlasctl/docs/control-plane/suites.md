# Suites

Generated from suite registries (`pyproject.toml` + `src/atlasctl/registry/suites.py`).

- Default suite: `refgrade`

## Configured Suites

- `ci`: includes=['refgrade'] items=2 complete=False
- `fast`: includes=[] items=4 complete=False
- `ops`: includes=[] items=2 complete=False
- `refgrade`: includes=['fast'] items=7 complete=True
- `refgrade_proof`: includes=['ci'] items=7 complete=True
- `release_0_1`: includes=['refgrade_proof'] items=3 complete=True

## First-Class Suites

- `all`: checks=214 markers=['docs', 'dev', 'ops', 'policies', 'configs']
- `ci`: checks=214 markers=['refgrade_required']
- `configs`: checks=2 markers=['configs']
- `dev`: checks=21 markers=['dev']
- `docs`: checks=17 markers=['docs']
- `internal`: checks=0 markers=['internal', 'internal-only']
- `local`: checks=212 markers=['fast']
- `ops`: checks=7 markers=['ops']
- `policies`: checks=167 markers=['policies']
- `refgrade`: checks=214 markers=['refgrade_required']
- `refgrade_proof`: checks=214 markers=['refgrade_required']
- `slow`: checks=2 markers=['slow']
