//! GRC Commands
//!
//! Tauri commands for GRC (Governance, Risk, Compliance) operations.

use crate::db::Database;
use crate::grc::{
    models::*,
    frameworks::{get_framework_controls, get_available_frameworks, get_framework_categories, FrameworkInfo, CategoryInfo},
    repository::{AssessmentRepository, ControlAssessmentRepository, EvidenceRepository},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

// ============================================================================
// Framework Commands
// ============================================================================

/// Get list of available frameworks
#[tauri::command]
pub async fn list_frameworks() -> Result<Vec<FrameworkInfo>, String> {
    Ok(get_available_frameworks())
}

/// Get all controls for a specific framework
#[tauri::command]
pub async fn get_framework_controls_cmd(framework: String) -> Result<Vec<Control>, String> {
    let fw = parse_framework_param(&framework)?;
    Ok(get_framework_controls(fw))
}

// ============================================================================
// Assessment Commands
// ============================================================================

/// Create assessment request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssessmentRequest {
    pub client_id: String,
    pub name: String,
    pub description: Option<String>,
    pub framework: String,
    pub scope: Option<String>,
    pub lead_assessor: String,
}

/// Create a new assessment
#[tauri::command]
pub async fn create_assessment(
    db: State<'_, Database>,
    request: CreateAssessmentRequest,
) -> Result<Assessment, String> {
    let framework = parse_framework_param(&request.framework)?;

    let assessment = Assessment {
        id: Uuid::new_v4().to_string(),
        client_id: request.client_id,
        name: request.name,
        description: request.description,
        framework,
        scope: request.scope,
        started_at: Utc::now(),
        completed_at: None,
        lead_assessor: request.lead_assessor,
        status: AssessmentStatus::Draft,
    };

    let repo = AssessmentRepository::new(&db);
    repo.create(&assessment).map_err(|e| e.to_string())?;

    Ok(assessment)
}

/// Get assessment by ID
#[tauri::command]
pub async fn get_assessment(
    db: State<'_, Database>,
    id: String,
) -> Result<Option<Assessment>, String> {
    let repo = AssessmentRepository::new(&db);
    repo.get(&id).map_err(|e| e.to_string())
}

/// List assessments for a client
#[tauri::command]
pub async fn list_client_assessments(
    db: State<'_, Database>,
    client_id: String,
) -> Result<Vec<Assessment>, String> {
    let repo = AssessmentRepository::new(&db);
    repo.list_by_client(&client_id).map_err(|e| e.to_string())
}

/// List all assessments
#[tauri::command]
pub async fn list_assessments(
    db: State<'_, Database>,
) -> Result<Vec<Assessment>, String> {
    let repo = AssessmentRepository::new(&db);
    repo.list_all().map_err(|e| e.to_string())
}

/// Update assessment status
#[tauri::command]
pub async fn update_assessment_status(
    db: State<'_, Database>,
    id: String,
    status: String,
) -> Result<bool, String> {
    let status = parse_assessment_status_param(&status)?;
    let repo = AssessmentRepository::new(&db);
    repo.update_status(&id, status).map_err(|e| e.to_string())
}

/// Delete assessment
#[tauri::command]
pub async fn delete_assessment(
    db: State<'_, Database>,
    id: String,
) -> Result<bool, String> {
    let repo = AssessmentRepository::new(&db);
    repo.delete(&id).map_err(|e| e.to_string())
}

// ============================================================================
// Control Assessment Commands
// ============================================================================

/// Update control assessment request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateControlAssessmentRequest {
    pub assessment_id: String,
    pub control_id: String,
    pub status: String,
    pub notes: Option<String>,
    pub gap_description: Option<String>,
    pub remediation: Option<String>,
    pub remediation_target: Option<String>,
    pub risk_rating: Option<u8>,
    pub assessed_by: String,
}

/// Update a control's assessment status
#[tauri::command]
pub async fn update_control_assessment(
    db: State<'_, Database>,
    request: UpdateControlAssessmentRequest,
) -> Result<ControlAssessment, String> {
    let status = parse_compliance_status_param(&request.status)?;

    let remediation_target = request.remediation_target
        .map(|s| chrono::DateTime::parse_from_rfc3339(&s)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| format!("Invalid date: {}", e)))
        .transpose()?;

    let ca = ControlAssessment {
        id: Uuid::new_v4().to_string(),
        assessment_id: request.assessment_id,
        control_id: request.control_id,
        status,
        notes: request.notes,
        gap_description: request.gap_description,
        remediation: request.remediation,
        remediation_target,
        risk_rating: request.risk_rating,
        evidence_ids: vec![],
        assessed_at: Utc::now(),
        assessed_by: request.assessed_by,
    };

    let repo = ControlAssessmentRepository::new(&db);
    repo.upsert(&ca).map_err(|e| e.to_string())?;

    Ok(ca)
}

/// Get all control assessments for an assessment
#[tauri::command]
pub async fn get_control_assessments(
    db: State<'_, Database>,
    assessment_id: String,
) -> Result<Vec<ControlAssessment>, String> {
    let repo = ControlAssessmentRepository::new(&db);
    repo.get_by_assessment(&assessment_id).map_err(|e| e.to_string())
}

/// Batch update control assessments request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateControlsRequest {
    pub assessment_id: String,
    pub control_ids: Vec<String>,
    pub status: String,
    pub assessed_by: String,
}

/// Batch update multiple controls at once
#[tauri::command]
pub async fn batch_update_controls(
    db: State<'_, Database>,
    request: BatchUpdateControlsRequest,
) -> Result<usize, String> {
    let status = parse_compliance_status_param(&request.status)?;
    let repo = ControlAssessmentRepository::new(&db);

    let mut updated = 0;
    for control_id in &request.control_ids {
        let ca = ControlAssessment {
            id: Uuid::new_v4().to_string(),
            assessment_id: request.assessment_id.clone(),
            control_id: control_id.clone(),
            status,
            notes: None,
            gap_description: None,
            remediation: None,
            remediation_target: None,
            risk_rating: None,
            evidence_ids: vec![],
            assessed_at: Utc::now(),
            assessed_by: request.assessed_by.clone(),
        };
        repo.upsert(&ca).map_err(|e| e.to_string())?;
        updated += 1;
    }

    Ok(updated)
}

// ============================================================================
// Evidence Commands
// ============================================================================

/// Create evidence request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEvidenceRequest {
    pub assessment_id: String,
    pub control_ids: Vec<String>,
    pub evidence_type: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub collected_by: String,
}

/// Add evidence to an assessment
#[tauri::command]
pub async fn create_evidence(
    db: State<'_, Database>,
    request: CreateEvidenceRequest,
) -> Result<Evidence, String> {
    let evidence_type = parse_evidence_type_param(&request.evidence_type)?;

    let evidence = Evidence {
        id: Uuid::new_v4().to_string(),
        assessment_id: request.assessment_id,
        control_ids: request.control_ids,
        evidence_type,
        title: request.title,
        description: request.description,
        file_path: request.file_path,
        url: request.url,
        file_hash: None, // TODO: Calculate hash if file provided
        collected_at: Utc::now(),
        collected_by: request.collected_by,
        notes: request.notes,
    };

    let repo = EvidenceRepository::new(&db);
    repo.create(&evidence).map_err(|e| e.to_string())?;

    Ok(evidence)
}

/// Get all evidence for an assessment
#[tauri::command]
pub async fn get_assessment_evidence(
    db: State<'_, Database>,
    assessment_id: String,
) -> Result<Vec<Evidence>, String> {
    let repo = EvidenceRepository::new(&db);
    repo.get_by_assessment(&assessment_id).map_err(|e| e.to_string())
}

/// Delete evidence
#[tauri::command]
pub async fn delete_evidence(
    db: State<'_, Database>,
    id: String,
) -> Result<bool, String> {
    let repo = EvidenceRepository::new(&db);
    repo.delete(&id).map_err(|e| e.to_string())
}

// ============================================================================
// Summary & Analytics Commands
// ============================================================================

/// Get assessment summary with compliance scores
#[tauri::command]
pub async fn get_assessment_summary(
    db: State<'_, Database>,
    assessment_id: String,
) -> Result<AssessmentSummary, String> {
    let assessment_repo = AssessmentRepository::new(&db);
    let assessment = assessment_repo
        .get(&assessment_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Assessment not found".to_string())?;

    let controls = get_framework_controls(assessment.framework);
    let control_repo = ControlAssessmentRepository::new(&db);
    let assessments = control_repo.get_by_assessment(&assessment_id).map_err(|e| e.to_string())?;

    let evidence_repo = EvidenceRepository::new(&db);
    let evidence_count = evidence_repo.count_by_assessment(&assessment_id).map_err(|e| e.to_string())?;

    // Build assessment map
    let assessment_map: std::collections::HashMap<String, ControlAssessment> = assessments
        .into_iter()
        .map(|ca| (ca.control_id.clone(), ca))
        .collect();

    // Calculate overall stats
    let mut compliant = 0;
    let mut partially_compliant = 0;
    let mut non_compliant = 0;
    let mut not_assessed = 0;
    let mut not_applicable = 0;
    let mut high_risk_gaps = 0;

    // Category breakdown
    let mut category_stats: std::collections::HashMap<String, (String, String, usize, usize, usize, usize, usize, usize)> =
        std::collections::HashMap::new();

    for control in &controls {
        let status = assessment_map
            .get(&control.id)
            .map(|ca| ca.status)
            .unwrap_or(ComplianceStatus::NotAssessed);

        let risk = assessment_map
            .get(&control.id)
            .and_then(|ca| ca.risk_rating);

        match status {
            ComplianceStatus::Compliant => compliant += 1,
            ComplianceStatus::PartiallyCompliant => partially_compliant += 1,
            ComplianceStatus::NonCompliant => {
                non_compliant += 1;
                if risk.unwrap_or(0) >= 4 {
                    high_risk_gaps += 1;
                }
            }
            ComplianceStatus::NotAssessed => not_assessed += 1,
            ComplianceStatus::NotApplicable => not_applicable += 1,
        }

        // Update category stats
        let category = &control.category;
        let (cat_name, cat_color) = get_category_info(assessment.framework, category);

        let entry = category_stats
            .entry(category.clone())
            .or_insert((cat_name, cat_color, 0, 0, 0, 0, 0, 0));

        entry.2 += 1; // total
        match status {
            ComplianceStatus::Compliant => entry.3 += 1,
            ComplianceStatus::PartiallyCompliant => entry.4 += 1,
            ComplianceStatus::NonCompliant => entry.5 += 1,
            ComplianceStatus::NotAssessed => entry.6 += 1,
            ComplianceStatus::NotApplicable => entry.7 += 1,
        }
    }

    // Calculate category scores
    let category_scores: Vec<CategoryScore> = category_stats
        .into_iter()
        .map(|(cat, (name, color, total, comp, partial, non_comp, not_ass, na))| {
            let applicable = total - na;
            let score = if applicable > 0 {
                ((comp as f64 + partial as f64 * 0.5) / applicable as f64) * 100.0
            } else {
                100.0
            };

            CategoryScore {
                category: cat,
                display_name: name,
                color,
                total_controls: total,
                compliant: comp,
                partially_compliant: partial,
                non_compliant: non_comp,
                not_assessed: not_ass,
                not_applicable: na,
                compliance_percentage: (score * 10.0).round() / 10.0,
            }
        })
        .collect();

    // Calculate overall compliance
    let total = controls.len();
    let applicable = total - not_applicable;
    let overall_compliance = if applicable > 0 {
        ((compliant as f64 + partially_compliant as f64 * 0.5) / applicable as f64) * 100.0
    } else {
        100.0
    };

    Ok(AssessmentSummary {
        assessment_id,
        framework: assessment.framework,
        overall_compliance: (overall_compliance * 10.0).round() / 10.0,
        total_controls: total,
        compliant,
        partially_compliant,
        non_compliant,
        not_assessed,
        not_applicable,
        category_scores,
        high_risk_gaps,
        evidence_count,
    })
}

// ============================================================================
// Compliance Status Command (Task A)
// ============================================================================

/// Get compliance status for a specific framework
/// Returns the overall completion and compliance percentages for all NIST CSF categories
#[tauri::command]
pub async fn get_compliance_status(
    db: State<'_, Database>,
    framework: String,
    client_id: Option<String>,
) -> Result<ComplianceStatusReport, String> {
    let fw = parse_framework_param(&framework)?;
    let controls = get_framework_controls(fw);
    let categories = get_framework_categories(fw);

    // Get all assessments for this framework
    let assessment_repo = AssessmentRepository::new(&db);
    let control_repo = ControlAssessmentRepository::new(&db);

    let assessments = if let Some(ref cid) = client_id {
        assessment_repo.list_by_client(cid).map_err(|e| e.to_string())?
    } else {
        assessment_repo.list_all().map_err(|e| e.to_string())?
    };

    // Filter to only this framework
    let framework_assessments: Vec<_> = assessments
        .into_iter()
        .filter(|a| a.framework == fw)
        .collect();

    // Collect all control assessments across all assessments
    let mut all_control_assessments: HashMap<String, ControlAssessment> = HashMap::new();
    for assessment in &framework_assessments {
        let cas = control_repo.get_by_assessment(&assessment.id).map_err(|e| e.to_string())?;
        for ca in cas {
            // Keep the most recent assessment for each control
            all_control_assessments.insert(ca.control_id.clone(), ca);
        }
    }

    // Calculate overall stats
    let mut total_assessed = 0;
    let mut total_compliant = 0;
    let mut total_partial = 0;
    let mut total_non_compliant = 0;
    let mut total_na = 0;

    // Build category breakdown
    let mut category_map: HashMap<String, (usize, usize, usize, usize, usize, usize)> = HashMap::new();

    for control in &controls {
        let status = all_control_assessments
            .get(&control.id)
            .map(|ca| ca.status)
            .unwrap_or(ComplianceStatus::NotAssessed);

        let entry = category_map.entry(control.category.clone()).or_insert((0, 0, 0, 0, 0, 0));
        entry.0 += 1; // total

        match status {
            ComplianceStatus::NotAssessed => {
                // Not assessed - counts towards total but not assessed count
            }
            ComplianceStatus::Compliant => {
                total_assessed += 1;
                total_compliant += 1;
                entry.1 += 1; // assessed
                entry.2 += 1; // compliant
            }
            ComplianceStatus::PartiallyCompliant => {
                total_assessed += 1;
                total_partial += 1;
                entry.1 += 1;
                entry.3 += 1; // partial
            }
            ComplianceStatus::NonCompliant => {
                total_assessed += 1;
                total_non_compliant += 1;
                entry.1 += 1;
                entry.4 += 1; // non-compliant
            }
            ComplianceStatus::NotApplicable => {
                total_assessed += 1;
                total_na += 1;
                entry.1 += 1;
                entry.5 += 1; // n/a
            }
        }
    }

    // Build category breakdown response
    let category_breakdown: Vec<CategoryComplianceStatus> = categories
        .iter()
        .map(|cat| {
            let stats = category_map.get(&cat.code).copied().unwrap_or((0, 0, 0, 0, 0, 0));
            let (total, assessed, compliant, partial, non_comp, na) = stats;

            let completion_pct = if total > 0 {
                (assessed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let applicable = assessed - na;
            let compliance_pct = if applicable > 0 {
                ((compliant as f64 + partial as f64 * 0.5) / applicable as f64) * 100.0
            } else {
                0.0
            };

            CategoryComplianceStatus {
                code: cat.code.clone(),
                name: cat.name.clone(),
                description: cat.description.clone(),
                color: cat.color.clone(),
                total_controls: total,
                assessed_controls: assessed,
                compliant,
                partially_compliant: partial,
                non_compliant: non_comp,
                completion_percentage: (completion_pct * 10.0).round() / 10.0,
                compliance_percentage: (compliance_pct * 10.0).round() / 10.0,
            }
        })
        .collect();

    let total_controls = controls.len();
    let completion_percentage = if total_controls > 0 {
        (total_assessed as f64 / total_controls as f64) * 100.0
    } else {
        0.0
    };

    let applicable = total_assessed - total_na;
    let compliance_percentage = if applicable > 0 {
        ((total_compliant as f64 + total_partial as f64 * 0.5) / applicable as f64) * 100.0
    } else {
        0.0
    };

    Ok(ComplianceStatusReport {
        framework: fw,
        completion_percentage: (completion_percentage * 10.0).round() / 10.0,
        compliance_percentage: (compliance_percentage * 10.0).round() / 10.0,
        total_controls,
        assessed_controls: total_assessed,
        compliant_controls: total_compliant,
        partially_compliant_controls: total_partial,
        non_compliant_controls: total_non_compliant,
        not_applicable_controls: total_na,
        category_breakdown,
        network_health_score: None,
        total_assets: None,
        last_updated: Utc::now().to_rfc3339(),
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_framework_param(s: &str) -> Result<Framework, String> {
    match s.to_uppercase().as_str() {
        "NIST_CSF_2" | "NISTCSF2" | "NIST_CSF2" | "NIST CSF 2.0" => Ok(Framework::NistCsf2),
        "SOC_2_TYPE_II" | "SOC2TYPEII" | "SOC2" | "SOC 2 TYPE II" => Ok(Framework::Soc2TypeII),
        "GDPR" => Ok(Framework::Gdpr),
        _ => Err(format!("Unknown framework: {}", s)),
    }
}

fn parse_assessment_status_param(s: &str) -> Result<AssessmentStatus, String> {
    match s.to_uppercase().as_str() {
        "DRAFT" => Ok(AssessmentStatus::Draft),
        "IN_PROGRESS" | "INPROGRESS" => Ok(AssessmentStatus::InProgress),
        "UNDER_REVIEW" | "UNDERREVIEW" => Ok(AssessmentStatus::UnderReview),
        "COMPLETED" => Ok(AssessmentStatus::Completed),
        "ARCHIVED" => Ok(AssessmentStatus::Archived),
        _ => Err(format!("Unknown assessment status: {}", s)),
    }
}

fn parse_compliance_status_param(s: &str) -> Result<ComplianceStatus, String> {
    match s.to_uppercase().as_str() {
        "NOT_ASSESSED" | "NOTASSESSED" => Ok(ComplianceStatus::NotAssessed),
        "COMPLIANT" => Ok(ComplianceStatus::Compliant),
        "PARTIALLY_COMPLIANT" | "PARTIALLYCOMPLIANT" => Ok(ComplianceStatus::PartiallyCompliant),
        "NON_COMPLIANT" | "NONCOMPLIANT" => Ok(ComplianceStatus::NonCompliant),
        "NOT_APPLICABLE" | "NOTAPPLICABLE" | "N/A" => Ok(ComplianceStatus::NotApplicable),
        _ => Err(format!("Unknown compliance status: {}", s)),
    }
}

fn parse_evidence_type_param(s: &str) -> Result<EvidenceType, String> {
    match s.to_uppercase().as_str() {
        "DOCUMENT" => Ok(EvidenceType::Document),
        "SCREENSHOT" => Ok(EvidenceType::Screenshot),
        "CONFIGURATION" => Ok(EvidenceType::Configuration),
        "SCAN_RESULT" | "SCANRESULT" => Ok(EvidenceType::ScanResult),
        "INTERVIEW" => Ok(EvidenceType::Interview),
        "LOG_FILE" | "LOGFILE" => Ok(EvidenceType::LogFile),
        "OTHER" => Ok(EvidenceType::Other),
        _ => Err(format!("Unknown evidence type: {}", s)),
    }
}

fn get_category_info(framework: Framework, category: &str) -> (String, String) {
    match framework {
        Framework::NistCsf2 => {
            match category {
                "GV" => ("Govern".to_string(), "#8b5cf6".to_string()),
                "ID" => ("Identify".to_string(), "#3b82f6".to_string()),
                "PR" => ("Protect".to_string(), "#22c55e".to_string()),
                "DE" => ("Detect".to_string(), "#f59e0b".to_string()),
                "RS" => ("Respond".to_string(), "#ef4444".to_string()),
                "RC" => ("Recover".to_string(), "#06b6d4".to_string()),
                _ => (category.to_string(), "#64748b".to_string()),
            }
        }
        Framework::Soc2TypeII => {
            match category {
                "CC" => ("Security".to_string(), "#3b82f6".to_string()),
                "A" => ("Availability".to_string(), "#22c55e".to_string()),
                "PI" => ("Processing Integrity".to_string(), "#f59e0b".to_string()),
                "C" => ("Confidentiality".to_string(), "#8b5cf6".to_string()),
                "P" => ("Privacy".to_string(), "#ec4899".to_string()),
                _ => (category.to_string(), "#64748b".to_string()),
            }
        }
        Framework::Gdpr => {
            match category {
                "CH2" => ("Principles".to_string(), "#3b82f6".to_string()),
                "CH3" => ("Data Subject Rights".to_string(), "#22c55e".to_string()),
                "CH4" => ("Controller & Processor".to_string(), "#f59e0b".to_string()),
                "CH5" => ("Transfers".to_string(), "#8b5cf6".to_string()),
                "CH6" => ("Supervisory Authorities".to_string(), "#ec4899".to_string()),
                "CH8" => ("Remedies".to_string(), "#ef4444".to_string()),
                _ => (category.to_string(), "#64748b".to_string()),
            }
        }
    }
}
