# Makefile Atlasctl Inventory

Current Makefiles that still reference `atlasctl` (repo-wide inventory for migration planning):

- `makefiles/_macros.mk`
- `makefiles/ci.mk`
- `makefiles/policies.mk`
- `makefiles/product.mk`
- `makefiles/root.mk`
- `makefiles/verification.mk` (comment/example text)

## Notes

- `makefiles/ops.mk`, `makefiles/docs.mk`, and `makefiles/configs.mk` are intended to stay thin `bijux dev atlas ...` wrappers.
- `root.mk` remains a large legacy surface and should be migrated in smaller namespace-specific batches (`ops`, `docs`, `configs`, `policies`, `reporting`).
