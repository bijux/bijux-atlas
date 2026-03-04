# Replication Metrics Dashboards

## Dashboard Panels

- replica groups total: `atlas_replica_groups_total`
- healthy replica groups: `atlas_replica_healthy_groups_total`
- average lag milliseconds: `atlas_replication_lag_ms_avg`
- replication throughput rows per second: `atlas_replication_throughput_rows_per_second`
- replica failures total: `atlas_replica_failures_total`

## Alert Hints

- lag warning: `atlas_replication_lag_ms_avg > 1000`
- lag critical: `atlas_replication_lag_ms_avg > 2000`
- health critical:
  `atlas_replica_healthy_groups_total < atlas_replica_groups_total`
