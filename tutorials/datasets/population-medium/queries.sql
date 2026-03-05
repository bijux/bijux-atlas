-- named_query: record_count
SELECT COUNT(*) AS record_count FROM population_medium;

-- named_query: sample_rows
SELECT * FROM population_medium LIMIT 10;
