#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawGeneRow {
    pub gene_id: String,
    pub name: Option<String>,
    pub seqid: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub biotype: Option<String>,
    pub transcript_count: Option<i64>,
    pub sequence_length: Option<i64>,
}

impl RawGeneRow {
    pub fn from_sql_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            gene_id: row.get::<_, String>(0)?,
            name: row.get::<_, Option<String>>(1)?,
            seqid: row.get::<_, Option<String>>(2)?,
            start: row.get::<_, Option<i64>>(3)?,
            end: row.get::<_, Option<i64>>(4)?,
            biotype: row.get::<_, Option<String>>(5)?,
            transcript_count: row.get::<_, Option<i64>>(6)?,
            sequence_length: row.get::<_, Option<i64>>(7)?,
        })
    }
}
