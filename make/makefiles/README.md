# Makefiles

Owner: `build-and-release`

## Contract
- Makefiles are delegation only.
- Make is a thin wrapper layer.
- Wrapper recipes delegate to stable `cargo ...` or dev control-plane surfaces.
- Public target surface is defined by make metadata and rendered by `make help`.
- `make/makefiles/root.mk` is the publish/router file, not a second control-plane.

See `make/makefiles/CONTRACT.md` for normative rules.

## Canonical wrapper files
- `make/makefiles/dev.mk`
- `make/makefiles/ci.mk`
- `make/makefiles/_docs.mk`
- `make/makefiles/_ops.mk`
- `make/makefiles/_configs.mk`
- `make/makefiles/docker.mk`
- `make/makefiles/_policies.mk`

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
