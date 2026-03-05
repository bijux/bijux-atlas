-- named_query: record_count
SELECT COUNT(*) AS record_count FROM genes_baseline;

-- named_query: sample_rows
SELECT * FROM genes_baseline LIMIT 10;
