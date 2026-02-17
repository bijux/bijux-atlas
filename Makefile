SHELL := /bin/sh

JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/root.mk
include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/docs.mk
include makefiles/ops.mk
include makefiles/policies.mk

.DEFAULT_GOAL := help
