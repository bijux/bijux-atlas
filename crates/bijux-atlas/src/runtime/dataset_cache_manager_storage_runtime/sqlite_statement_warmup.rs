fn prime_prepared_statements(conn: &Connection) {
    let hot_sql = [
        "SELECT gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length FROM gene_summary WHERE gene_id = ?1 ORDER BY gene_id LIMIT ?2",
        "SELECT gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length FROM gene_summary WHERE biotype = ?1 ORDER BY gene_id LIMIT ?2",
        "SELECT g.gene_id, g.name, g.seqid, g.start, g.end, g.biotype, g.transcript_count, g.sequence_length FROM gene_summary g JOIN gene_summary_rtree r ON r.gene_rowid = g.id WHERE g.seqid = ?1 AND r.start <= ?2 AND r.end >= ?3 ORDER BY g.seqid, g.start, g.gene_id LIMIT ?4",
        "SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary WHERE transcript_id=?1 LIMIT 1",
        "SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary WHERE parent_gene_id = ? ORDER BY seqid ASC, start ASC, transcript_id ASC LIMIT ?",
        "SELECT seqid,start,end FROM gene_summary WHERE gene_id = ?1 LIMIT 1",
    ];
    for sql in hot_sql {
        let _ = conn.prepare_cached(sql);
    }
}
