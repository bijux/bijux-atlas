# Layout shell-to-python migration checklist

This checklist tracks migration of legacy shell checks under
`packages/atlasctl/src/atlasctl/checks/layout/shell/`.

## Converted to Python (`checks/layout/root`)
- [x] `check_root_shape.sh` -> `root/check_root_shape.py` (`repo.root_shape`)
- [x] `check_no_forbidden_paths.sh` -> `root/check_forbidden_paths.py` (`repo.no_forbidden_paths`)
- [x] `check_no_direct_script_runs.sh` -> `root/check_no_direct_script_runs.py` (`repo.no_direct_script_runs`)
- [x] `check_root_determinism.sh` -> `root/check_root_determinism.py` (`repo.root_determinism`)

## Remaining shell checks (to migrate)
- [ ] artifacts and ops helper checks under `checks/layout/shell/`
- [ ] repo hygiene shell checks under `checks/layout/shell/`
- [ ] docs/scripts drift shell checks under `checks/layout/shell/`

## Policy
- New policy-critical checks must land as Python modules under `checks/layout/*`.
- Existing shell checks remain transitional and must keep strict shell headers.
