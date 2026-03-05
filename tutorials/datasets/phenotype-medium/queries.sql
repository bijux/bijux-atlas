-- named_query: record_count
SELECT COUNT(*) AS record_count FROM phenotype_medium;

-- named_query: sample_rows
SELECT * FROM phenotype_medium LIMIT 10;
