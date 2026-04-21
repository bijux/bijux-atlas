install:
	@echo "release bundle prerequisites are handled by Atlas make targets"

build:
	@$(MAKE) -f makes/root.mk -C . dist

.PHONY: install build
