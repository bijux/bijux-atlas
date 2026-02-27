# Makefiles

Owner: `build-and-release`

## Contract
- Makefiles are delegation only.
- Make is a thin wrapper layer.
- Wrapper recipes delegate to stable `cargo ...` or dev control-plane surfaces.
- Public target surface is defined by make metadata and rendered by `make help`.
- `makefiles/root.mk` is the publish/router file, not a second control-plane.

See `makefiles/CONTRACT.md` for normative rules.

## Canonical wrapper files
- `makefiles/dev.mk`
- `makefiles/ci.mk`
- `makefiles/_docs.mk`
- `makefiles/_ops.mk`
- `makefiles/_configs.mk`
- `makefiles/docker.mk`
- `makefiles/_policies.mk`

## Minimal public UX
- `make help`
- `make list`
- `make dev-atlas`
- `make ci`
- `make ci-pr`
- `make fmt`
- `make lint`
- `make test`
- `make coverage`
- `make check`
- `make check-list`
- `make gates`
- `make audit`
- `make docs`
- `make configs`
- `make ops`
- `make policies`
- `make doctor`
- `make clean`

## Verification
```bash
make ops-doctor
make help
cargo run -q -p bijux-dev-atlas -- check run --domain make
```
