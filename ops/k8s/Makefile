SHELL := /bin/sh

.PHONY: tests templates report pod-churn

tests:
	@$(MAKE) -C ../.. ops-k8s-tests

templates:
	@$(MAKE) -C ../.. ops-k8s-template-tests

report:
	@./tests/report.sh

pod-churn:
	@../../ops/e2e/k8s/tests/pod-churn.sh
