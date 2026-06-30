use super::*;

fn split_source_ids(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn wiki_decision_ledger_entry_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<WikiDecisionLedgerEntry> {
    let reviewed_source_card_ids: String = row.get("reviewed_source_card_ids")?;
    let source_count: i64 = row.get("source_count")?;
    Ok(WikiDecisionLedgerEntry {
        page_id: row.get("page_id")?,
        page_title: row.get("page_title")?,
        decision: row.get("decision")?,
        reviewed_source_card_ids: split_source_ids(&reviewed_source_card_ids),
        source_count: source_count.max(0) as usize,
        rationale: row.get("rationale")?,
        follow_up: row.get("follow_up")?,
        reviewed_at: row.get("reviewed_at")?,
        first_seen_at: row.get("first_seen_at")?,
        updated_at: row.get("updated_at")?,
        source_file: row.get("source_file")?,
    })
}

impl Store {
    pub fn wiki_decision_ledger_summary(&self) -> Result<WikiDecisionLedgerSummary> {
        let totals = self.conn.query_row(
            r#"
            SELECT COUNT(*) AS rows,
                   COUNT(DISTINCT page_id) AS pages,
                   MAX(reviewed_at) AS newest_reviewed_at,
                   MIN(reviewed_at) AS oldest_reviewed_at
            FROM wiki_editorial_decision_ledger
            "#,
            [],
            |row| {
                Ok((
                    row.get::<_, i64>("rows")?,
                    row.get::<_, i64>("pages")?,
                    row.get::<_, Option<String>>("newest_reviewed_at")?,
                    row.get::<_, Option<String>>("oldest_reviewed_at")?,
                ))
            },
        )?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT decision, COUNT(*) AS count
            FROM wiki_editorial_decision_ledger
            GROUP BY decision
            ORDER BY decision
            "#,
        )?;
        let mut decision_counts = BTreeMap::new();
        for row in rows(stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>("decision")?,
                row.get::<_, i64>("count")?,
            ))
        })?)? {
            decision_counts.insert(row.0, row.1.max(0) as usize);
        }
        Ok(WikiDecisionLedgerSummary {
            rows: totals.0.max(0) as usize,
            pages: totals.1.max(0) as usize,
            decision_counts,
            newest_reviewed_at: totals.2,
            oldest_reviewed_at: totals.3,
        })
    }

    pub fn list_wiki_decision_ledger(&self, limit: usize) -> Result<Vec<WikiDecisionLedgerEntry>> {
        let bounded_limit = limit.clamp(1, 500);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT page_id, page_title, decision, reviewed_source_card_ids,
                   source_count, rationale, follow_up, reviewed_at,
                   first_seen_at, updated_at, source_file
            FROM wiki_editorial_decision_ledger
            ORDER BY reviewed_at DESC, page_title, page_id
            LIMIT ?1
            "#,
        )?;
        rows(stmt.query_map(
            params![bounded_limit as i64],
            wiki_decision_ledger_entry_from_row,
        )?)
    }
}
