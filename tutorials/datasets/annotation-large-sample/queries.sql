-- named_query: record_count
SELECT COUNT(*) AS record_count FROM annotation_large_sample;

-- named_query: sample_rows
SELECT * FROM annotation_large_sample LIMIT 10;
