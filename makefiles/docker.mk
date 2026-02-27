DOCKER_RUN_ID ?= $(RUN_ID)

docker: docker-validate

docker-validate:
	@$(DEV_ATLAS) docker validate --run-id $(DOCKER_RUN_ID)-$@ --format json

docker-build:
	@$(DEV_ATLAS) docker build --allow-subprocess --run-id $(DOCKER_RUN_ID)-$@ --format json

docker-smoke:
	@$(DEV_ATLAS) docker smoke --allow-subprocess --run-id $(DOCKER_RUN_ID)-$@ --format json

docker-sbom:
	@$(DEV_ATLAS) docker sbom --allow-subprocess --run-id $(DOCKER_RUN_ID)-$@ --format json

docker-scan:
	@$(DEV_ATLAS) docker scan --allow-subprocess --allow-network --run-id $(DOCKER_RUN_ID)-$@ --format json

docker-lock:
	@$(DEV_ATLAS) docker lock --allow-write --run-id $(DOCKER_RUN_ID)-$@ --format json

docker-release:
	@$(DEV_ATLAS) docker release --allow-subprocess --allow-network --i-know-what-im-doing --run-id $(DOCKER_RUN_ID)-$@ --format json

docker-gate:
	@$(MAKE) -s docker-validate
	@$(MAKE) -s docker-build
	@$(MAKE) -s docker-smoke
	@$(MAKE) -s docker-sbom
	@$(MAKE) -s docker-scan

.PHONY: docker docker-validate docker-build docker-smoke docker-sbom docker-scan docker-lock docker-release docker-gate
