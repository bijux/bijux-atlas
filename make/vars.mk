# Scope: shared variable defaults for the make wrapper surface.
# Public targets: none
ARTIFACT_ROOT ?= artifacts
FORMAT ?= text
GLOBAL_OUTPUT_FORMAT ?= $(if $(filter json,$(FORMAT)),json,human)
CONTRACTS_FORMAT ?= $(if $(filter json,$(FORMAT)),json,human)
REPO_ROOT := $(abspath .)
