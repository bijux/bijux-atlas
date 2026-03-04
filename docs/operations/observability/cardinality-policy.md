# Cardinality Policy

Cardinality controls keep metrics queryable and affordable.

Rules:

- No raw user or request ids in labels.
- No unbounded free-form text labels.
- New labels require explicit operational justification.
- Bucket or hash unbounded dimensions before labeling.

Failure mode:

- High-cardinality labels increase TSDB load and can hide real incidents.
