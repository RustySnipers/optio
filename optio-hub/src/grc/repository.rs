//! GRC Repository
//!
//! Database operations for GRC assessments, controls, and evidence.

use crate::db::Database;
use crate::error::{OptioError, OptioResult};
use crate::grc::models::*;
use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

/// Initialize GRC database schema
pub fn init_grc_schema(db: &Database) -> OptioResult<()> {
    let conn = db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

    conn.execute_batch(r#"
        -- Assessments table
        CREATE TABLE IF NOT EXISTS assessments (
            id TEXT PRIMARY KEY,
            client_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            framework TEXT NOT NULL,
            scope TEXT,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            lead_assessor TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'DRAFT',
            FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
        );

        -- Control assessments
        CREATE TABLE IF NOT EXISTS control_assessments (
            id TEXT PRIMARY KEY,
            assessment_id TEXT NOT NULL,
            control_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'NOT_ASSESSED',
            notes TEXT,
            gap_description TEXT,
            remediation TEXT,
            remediation_target TEXT,
            risk_rating INTEGER,
            assessed_at TEXT NOT NULL,
            assessed_by TEXT NOT NULL,
            FOREIGN KEY (assessment_id) REFERENCES assessments(id) ON DELETE CASCADE,
            UNIQUE(assessment_id, control_id)
        );

        -- Evidence
        CREATE TABLE IF NOT EXISTS evidence (
            id TEXT PRIMARY KEY,
            assessment_id TEXT NOT NULL,
            evidence_type TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            file_path TEXT,
            url TEXT,
            file_hash TEXT,
            collected_at TEXT NOT NULL,
            collected_by TEXT NOT NULL,
            notes TEXT,
            FOREIGN KEY (assessment_id) REFERENCES assessments(id) ON DELETE CASCADE
        );

        -- Evidence to control mapping (many-to-many)
        CREATE TABLE IF NOT EXISTS evidence_controls (
            evidence_id TEXT NOT NULL,
            control_id TEXT NOT NULL,
            PRIMARY KEY (evidence_id, control_id),
            FOREIGN KEY (evidence_id) REFERENCES evidence(id) ON DELETE CASCADE
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_assessments_client ON assessments(client_id);
        CREATE INDEX IF NOT EXISTS idx_assessments_framework ON assessments(framework);
        CREATE INDEX IF NOT EXISTS idx_control_assessments_assessment ON control_assessments(assessment_id);
        CREATE INDEX IF NOT EXISTS idx_evidence_assessment ON evidence(assessment_id);
    "#)?;

    tracing::info!("GRC database schema initialized");
    Ok(())
}

/// Assessment repository
pub struct AssessmentRepository<'a> {
    db: &'a Database,
}

impl<'a> AssessmentRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        AssessmentRepository { db }
    }

    pub fn create(&self, assessment: &Assessment) -> OptioResult<()> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        conn.execute(
            r#"INSERT INTO assessments
               (id, client_id, name, description, framework, scope, started_at, completed_at, lead_assessor, status)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                assessment.id,
                assessment.client_id,
                assessment.name,
                assessment.description,
                format!("{:?}", assessment.framework),
                assessment.scope,
                assessment.started_at.to_rfc3339(),
                assessment.completed_at.map(|d| d.to_rfc3339()),
                assessment.lead_assessor,
                format!("{:?}", assessment.status),
            ],
        )?;

        tracing::debug!("Created assessment: {}", assessment.id);
        Ok(())
    }

    pub fn get(&self, id: &str) -> OptioResult<Option<Assessment>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"SELECT id, client_id, name, description, framework, scope,
                      started_at, completed_at, lead_assessor, status
               FROM assessments WHERE id = ?1"#
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(parse_assessment_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_by_client(&self, client_id: &str) -> OptioResult<Vec<Assessment>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"SELECT id, client_id, name, description, framework, scope,
                      started_at, completed_at, lead_assessor, status
               FROM assessments WHERE client_id = ?1 ORDER BY started_at DESC"#
        )?;

        let assessments = stmt.query_map(params![client_id], |row| {
            Ok(parse_assessment_row(row).unwrap())
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(assessments)
    }

    pub fn list_all(&self) -> OptioResult<Vec<Assessment>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"SELECT id, client_id, name, description, framework, scope,
                      started_at, completed_at, lead_assessor, status
               FROM assessments ORDER BY started_at DESC"#
        )?;

        let assessments = stmt.query_map([], |row| {
            Ok(parse_assessment_row(row).unwrap())
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(assessments)
    }

    pub fn update_status(&self, id: &str, status: AssessmentStatus) -> OptioResult<bool> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let completed_at = if status == AssessmentStatus::Completed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        let updated = conn.execute(
            "UPDATE assessments SET status = ?2, completed_at = COALESCE(?3, completed_at) WHERE id = ?1",
            params![id, format!("{:?}", status), completed_at],
        )?;

        Ok(updated > 0)
    }

    pub fn delete(&self, id: &str) -> OptioResult<bool> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;
        let deleted = conn.execute("DELETE FROM assessments WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }
}

/// Control assessment repository
pub struct ControlAssessmentRepository<'a> {
    db: &'a Database,
}

impl<'a> ControlAssessmentRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        ControlAssessmentRepository { db }
    }

    pub fn upsert(&self, ca: &ControlAssessment) -> OptioResult<()> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        conn.execute(
            r#"INSERT INTO control_assessments
               (id, assessment_id, control_id, status, notes, gap_description,
                remediation, remediation_target, risk_rating, assessed_at, assessed_by)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
               ON CONFLICT(assessment_id, control_id) DO UPDATE SET
                   status = excluded.status,
                   notes = excluded.notes,
                   gap_description = excluded.gap_description,
                   remediation = excluded.remediation,
                   remediation_target = excluded.remediation_target,
                   risk_rating = excluded.risk_rating,
                   assessed_at = excluded.assessed_at,
                   assessed_by = excluded.assessed_by"#,
            params![
                ca.id,
                ca.assessment_id,
                ca.control_id,
                format!("{:?}", ca.status),
                ca.notes,
                ca.gap_description,
                ca.remediation,
                ca.remediation_target.map(|d| d.to_rfc3339()),
                ca.risk_rating,
                ca.assessed_at.to_rfc3339(),
                ca.assessed_by,
            ],
        )?;

        Ok(())
    }

    pub fn get_by_assessment(&self, assessment_id: &str) -> OptioResult<Vec<ControlAssessment>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"SELECT id, assessment_id, control_id, status, notes, gap_description,
                      remediation, remediation_target, risk_rating, assessed_at, assessed_by
               FROM control_assessments WHERE assessment_id = ?1"#
        )?;

        let assessments = stmt.query_map(params![assessment_id], |row| {
            Ok(parse_control_assessment_row(row).unwrap())
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(assessments)
    }

    pub fn get_by_control(&self, assessment_id: &str, control_id: &str) -> OptioResult<Option<ControlAssessment>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"SELECT id, assessment_id, control_id, status, notes, gap_description,
                      remediation, remediation_target, risk_rating, assessed_at, assessed_by
               FROM control_assessments WHERE assessment_id = ?1 AND control_id = ?2"#
        )?;

        let mut rows = stmt.query(params![assessment_id, control_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(parse_control_assessment_row(row)?))
        } else {
            Ok(None)
        }
    }
}

/// Evidence repository
pub struct EvidenceRepository<'a> {
    db: &'a Database,
}

impl<'a> EvidenceRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        EvidenceRepository { db }
    }

    pub fn create(&self, evidence: &Evidence) -> OptioResult<()> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        conn.execute(
            r#"INSERT INTO evidence
               (id, assessment_id, evidence_type, title, description, file_path,
                url, file_hash, collected_at, collected_by, notes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                evidence.id,
                evidence.assessment_id,
                format!("{:?}", evidence.evidence_type),
                evidence.title,
                evidence.description,
                evidence.file_path,
                evidence.url,
                evidence.file_hash,
                evidence.collected_at.to_rfc3339(),
                evidence.collected_by,
                evidence.notes,
            ],
        )?;

        // Link to controls
        for control_id in &evidence.control_ids {
            conn.execute(
                "INSERT OR IGNORE INTO evidence_controls (evidence_id, control_id) VALUES (?1, ?2)",
                params![evidence.id, control_id],
            )?;
        }

        Ok(())
    }

    pub fn get_by_assessment(&self, assessment_id: &str) -> OptioResult<Vec<Evidence>> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;

        let mut stmt = conn.prepare(
            r#"SELECT id, assessment_id, evidence_type, title, description, file_path,
                      url, file_hash, collected_at, collected_by, notes
               FROM evidence WHERE assessment_id = ?1 ORDER BY collected_at DESC"#
        )?;

        let evidence_list: Vec<Evidence> = stmt.query_map(params![assessment_id], |row| {
            Ok(parse_evidence_row(row, vec![]).unwrap())
        })?
        .filter_map(|r| r.ok())
        .collect();

        // Load control mappings
        let mut result = Vec::new();
        for mut ev in evidence_list {
            let mut ctrl_stmt = conn.prepare(
                "SELECT control_id FROM evidence_controls WHERE evidence_id = ?1"
            )?;
            let control_ids: Vec<String> = ctrl_stmt
                .query_map(params![ev.id], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            ev.control_ids = control_ids;
            result.push(ev);
        }

        Ok(result)
    }

    pub fn delete(&self, id: &str) -> OptioResult<bool> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;
        let deleted = conn.execute("DELETE FROM evidence WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    pub fn count_by_assessment(&self, assessment_id: &str) -> OptioResult<usize> {
        let conn = self.db.conn.lock().map_err(|e| OptioError::Database(e.to_string()))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM evidence WHERE assessment_id = ?1",
            params![assessment_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

// Helper functions for parsing rows

fn parse_assessment_row(row: &rusqlite::Row) -> OptioResult<Assessment> {
    let framework_str: String = row.get(4)?;
    let status_str: String = row.get(9)?;

    Ok(Assessment {
        id: row.get(0)?,
        client_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        framework: parse_framework(&framework_str)?,
        scope: row.get(5)?,
        started_at: parse_datetime(&row.get::<_, String>(6)?)?,
        completed_at: row.get::<_, Option<String>>(7)?
            .map(|s| parse_datetime(&s))
            .transpose()?,
        lead_assessor: row.get(8)?,
        status: parse_assessment_status(&status_str)?,
    })
}

fn parse_control_assessment_row(row: &rusqlite::Row) -> OptioResult<ControlAssessment> {
    let status_str: String = row.get(3)?;

    Ok(ControlAssessment {
        id: row.get(0)?,
        assessment_id: row.get(1)?,
        control_id: row.get(2)?,
        status: parse_compliance_status(&status_str)?,
        notes: row.get(4)?,
        gap_description: row.get(5)?,
        remediation: row.get(6)?,
        remediation_target: row.get::<_, Option<String>>(7)?
            .map(|s| parse_datetime(&s))
            .transpose()?,
        risk_rating: row.get(8)?,
        evidence_ids: vec![], // Loaded separately
        assessed_at: parse_datetime(&row.get::<_, String>(9)?)?,
        assessed_by: row.get(10)?,
    })
}

fn parse_evidence_row(row: &rusqlite::Row, control_ids: Vec<String>) -> OptioResult<Evidence> {
    let type_str: String = row.get(2)?;

    Ok(Evidence {
        id: row.get(0)?,
        assessment_id: row.get(1)?,
        evidence_type: parse_evidence_type(&type_str)?,
        title: row.get(3)?,
        description: row.get(4)?,
        file_path: row.get(5)?,
        url: row.get(6)?,
        file_hash: row.get(7)?,
        collected_at: parse_datetime(&row.get::<_, String>(8)?)?,
        collected_by: row.get(9)?,
        notes: row.get(10)?,
        control_ids,
    })
}

fn parse_framework(s: &str) -> OptioResult<Framework> {
    match s {
        "NistCsf2" => Ok(Framework::NistCsf2),
        "Soc2TypeII" => Ok(Framework::Soc2TypeII),
        "Gdpr" => Ok(Framework::Gdpr),
        _ => Err(OptioError::Database(format!("Unknown framework: {}", s))),
    }
}

fn parse_assessment_status(s: &str) -> OptioResult<AssessmentStatus> {
    match s {
        "Draft" => Ok(AssessmentStatus::Draft),
        "InProgress" => Ok(AssessmentStatus::InProgress),
        "UnderReview" => Ok(AssessmentStatus::UnderReview),
        "Completed" => Ok(AssessmentStatus::Completed),
        "Archived" => Ok(AssessmentStatus::Archived),
        _ => Err(OptioError::Database(format!("Unknown assessment status: {}", s))),
    }
}

fn parse_compliance_status(s: &str) -> OptioResult<ComplianceStatus> {
    match s {
        "NotAssessed" => Ok(ComplianceStatus::NotAssessed),
        "Compliant" => Ok(ComplianceStatus::Compliant),
        "PartiallyCompliant" => Ok(ComplianceStatus::PartiallyCompliant),
        "NonCompliant" => Ok(ComplianceStatus::NonCompliant),
        "NotApplicable" => Ok(ComplianceStatus::NotApplicable),
        _ => Err(OptioError::Database(format!("Unknown compliance status: {}", s))),
    }
}

fn parse_evidence_type(s: &str) -> OptioResult<EvidenceType> {
    match s {
        "Document" => Ok(EvidenceType::Document),
        "Screenshot" => Ok(EvidenceType::Screenshot),
        "Configuration" => Ok(EvidenceType::Configuration),
        "ScanResult" => Ok(EvidenceType::ScanResult),
        "Interview" => Ok(EvidenceType::Interview),
        "LogFile" => Ok(EvidenceType::LogFile),
        "Other" => Ok(EvidenceType::Other),
        _ => Err(OptioError::Database(format!("Unknown evidence type: {}", s))),
    }
}

fn parse_datetime(s: &str) -> OptioResult<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| OptioError::Database(format!("Invalid datetime: {}", e)))
}
