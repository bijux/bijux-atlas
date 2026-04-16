# Scope: canonical include hub for the repository makes surface.
# Public targets: none
SHELL := /bin/sh
.DEFAULT_GOAL := help
JOBS ?= auto
FAIL_FAST ?= 0
ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local
SUITE_FAIL_FAST_FLAG := $(if $(filter 1 true yes,$(FAIL_FAST)),--fail-fast,--no-fail-fast)

include makes/vars.mk
include makes/paths.mk
include makes/macros.mk
include makes/build.mk
include makes/cargo.mk
include makes/checks.mk
include makes/ci.mk
include makes/configs.mk
include makes/contracts.mk
include makes/dev.mk
include makes/docker.mk
include makes/docs.mk
include makes/bijux-docs.mk
include makes/bijux-std.mk
include makes/entrypoints.mk
include makes/gh.mk
include makes/k8s.mk
include makes/ops.mk
include makes/policies.mk
include makes/runenv.mk
include makes/verification.mk

SHELL := /bin/bash

CURATED_TARGETS := \
	build ci-fast ci-nightly ci-pr clean docker doctor help k8s-render k8s-validate kind-down kind-reset kind-status kind-up lint-make openapi-generate ops-contracts ops-contracts-effect registry-doctor release-plan release-verify root-surface-explain stack-down stack-up suites-list tests-all
