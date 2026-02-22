# Makefiles

Owner: `build-and-release`

## Contract
- Make is a thin wrapper layer.
- Wrapper recipes delegate to `./bin/atlasctl ...` only.
- Public target surface is defined by atlasctl make metadata and rendered by `make help`.
- `makefiles/root.mk` is the publish/router file, not a second control-plane.

See `makefiles/CONTRACT.md` for normative rules.

## Canonical wrapper files
- `makefiles/dev.mk`
- `makefiles/ci.mk`
- `makefiles/docs.mk`
- `makefiles/ops.mk`

Optional wrapper files (not part of default root include):
- `makefiles/policies.mk` (`bypass-report` only)
- `makefiles/scripts.mk` (dependency transition helpers only)

## Minimal public UX
- `make help`
- `make list`
- `make root`
- `make root-local`
- `make nightly`
- `make ci`
- `make fmt`
- `make lint`
- `make test`
- `make coverage`
- `make check`
- `make audit`
- `make docs`
- `make ops`
- `make k8s`
- `make load`
- `make obs`
- `make doctor`
- `make report`

## Verification
```bash
make atlasctl-check
make help
./bin/atlasctl check run make
```
