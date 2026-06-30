use crate::*;

pub(crate) fn ensure_proof_packet_schema_on(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS proof_packets (
          id TEXT PRIMARY KEY,
          scope TEXT NOT NULL,
          title TEXT NOT NULL,
          proof_level TEXT NOT NULL,
          status TEXT NOT NULL,
          summary TEXT NOT NULL,
          artifact_root TEXT,
          reviewer TEXT,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          promoted_at TEXT,
          CHECK(status IN ('draft', 'partial', 'blocked', 'passed', 'promoted', 'rejected'))
        );

        CREATE INDEX IF NOT EXISTS idx_proof_packets_scope_created
        ON proof_packets(scope, created_at DESC);

        CREATE TABLE IF NOT EXISTS proof_claims (
          id TEXT PRIMARY KEY,
          packet_id TEXT NOT NULL,
          claim_key TEXT NOT NULL,
          claim TEXT NOT NULL,
          status TEXT NOT NULL,
          proof_level TEXT NOT NULL,
          evidence_json TEXT NOT NULL DEFAULT '[]',
          refutation_json TEXT NOT NULL DEFAULT '[]',
          gates_json TEXT NOT NULL DEFAULT '[]',
          created_at TEXT NOT NULL,
          UNIQUE(packet_id, claim_key),
          CHECK(status IN ('proven', 'partial', 'blocked', 'refuted', 'not_claimed')),
          FOREIGN KEY(packet_id) REFERENCES proof_packets(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_proof_claims_packet_status
        ON proof_claims(packet_id, status);

        CREATE TABLE IF NOT EXISTS proof_artifacts (
          id TEXT PRIMARY KEY,
          packet_id TEXT NOT NULL,
          artifact_kind TEXT NOT NULL,
          label TEXT NOT NULL,
          path TEXT,
          sha256 TEXT,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          FOREIGN KEY(packet_id) REFERENCES proof_packets(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_proof_artifacts_packet
        ON proof_artifacts(packet_id);

        CREATE TABLE IF NOT EXISTS proof_checks (
          id TEXT PRIMARY KEY,
          packet_id TEXT NOT NULL,
          check_kind TEXT NOT NULL,
          command TEXT NOT NULL,
          status TEXT NOT NULL,
          exit_code INTEGER,
          duration_ms INTEGER,
          output_excerpt TEXT,
          metadata_json TEXT NOT NULL DEFAULT '{}',
          created_at TEXT NOT NULL,
          CHECK(status IN ('passed', 'failed', 'skipped', 'blocked')),
          FOREIGN KEY(packet_id) REFERENCES proof_packets(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_proof_checks_packet_status
        ON proof_checks(packet_id, status);
        "#,
    )?;
    Ok(())
}
