-- named_query: record_count
SELECT COUNT(*) AS record_count FROM assembly_large_sample;

-- named_query: sample_rows
SELECT * FROM assembly_large_sample LIMIT 10;
