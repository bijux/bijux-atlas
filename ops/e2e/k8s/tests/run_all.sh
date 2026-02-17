#!/usr/bin/env sh
set -eu

DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

for t in \
  test_values_contract.sh \
  test_defaults_safe.sh \
  test_helm_templates.sh \
  test_install.sh \
  test_networkpolicy.sh \
  test_secrets.sh \
  test_configmap.sh \
  test_cached_only_mode.sh \
  test_pdb.sh \
  test_hpa.sh \
  test_rollout.sh \
  test_rollback.sh \
  test_warmup_job.sh \
  test_catalog_publish_job.sh \
  test_readiness_semantics.sh \
  test_resource_limits.sh \
  test_offline_profile.sh
  do
  "$DIR/$t"
done
