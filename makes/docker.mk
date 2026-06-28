# Scope: docker wrapper target delegated to bijux-atlas-dev docker surfaces.
# Public targets: docker

docker: ## Run canonical docker validation through dev-atlas
	@printf '%s\n' "run: $(DEV_ATLAS) docker check --allow-subprocess --format $(FORMAT)"
	@$(DEV_ATLAS) docker check --allow-subprocess --format $(FORMAT)

.PHONY: docker
