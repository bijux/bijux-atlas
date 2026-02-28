# Make Contract

- `MAKE-DIR-001`: Only `make/README.md` and `make/CONTRACT.md` may exist as markdown under `make/`. Enforced by: `make.docs.allowed_root_docs_only`.
- `MAKE-DIR-002`: No markdown files may exist under `make/makefiles/`. Enforced by: `make.docs.no_nested_markdown`.
- `MAKE-DIR-003`: The top-level `make/` surface is fixed to curated wrapper files only. Enforced by: `make.surface.allowed_root_files`.
- `MAKE-ENV-001`: The make wrapper tree contains exactly one macros file and exactly one run-environment file. Enforced by: `make.env.single_macros_and_runenv`.
- `MAKE-ENV-002`: `make/macros.mk` contains only pure macro definitions, and `make/makefiles/_runenv.mk` contains only deterministic environment defaults and exports. Enforced by: `make.env.role_boundary`.
- `MAKE-INCLUDE-001`: The root `Makefile` includes exactly one file: `make/public.mk`. Enforced by: `make.includes.root_single_entrypoint`.
- `MAKE-INCLUDE-002`: `make/public.mk` includes only the approved wrapper modules: vars, paths, macros, phony, `_internal`, and checks. Enforced by: `make.includes.public_surface`.
- `MAKE-INCLUDE-003`: The include graph under `Makefile` and `make/**/*.mk` is acyclic. Enforced by: `make.includes.acyclic`.
