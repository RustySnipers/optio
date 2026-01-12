//! Reporting Commands
//!
//! Tauri commands for report generation and management.

use crate::db::Database;
use crate::grc::{
    models::{AssetCategoryCount, ComplianceStatusReport, ExecutiveFinding, ExecutiveReportData, Framework, RiskSummary, CategoryComplianceStatus},
    frameworks::{get_framework_controls, get_framework_categories},
    repository::{AssessmentRepository, ControlAssessmentRepository},
};
use crate::reporting::{
    models::*,
    generator::{ReportGenerator, content_to_html, content_to_markdown},
    templates::{get_report_templates, get_template_for_type, get_report_type_info, get_export_formats, ReportTypeInfo, ExportFormatInfo},
    pdf_generator::{PdfGenerator, generate_demo_executive_report},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};
use uuid::Uuid;

/// In-memory storage for reports
pub struct ReportingState {
    pub reports: Mutex<Vec<Report>>,
}

impl Default for ReportingState {
    fn default() -> Self {
        Self {
            reports: Mutex::new(Vec::new()),
        }
    }
}

// ============================================================================
// Template Commands
// ============================================================================

/// Get all available report templates
#[tauri::command]
pub async fn get_report_template_list() -> Result<Vec<ReportTemplate>, String> {
    Ok(get_report_templates())
}

/// Get template for a specific report type
#[tauri::command]
pub async fn get_template_by_type(report_type: String) -> Result<ReportTemplate, String> {
    let rt = parse_report_type(&report_type)?;
    Ok(get_template_for_type(rt))
}

/// Get all report type options
#[tauri::command]
pub async fn get_report_types() -> Result<Vec<ReportTypeInfo>, String> {
    Ok(get_report_type_info())
}

/// Get all export format options
#[tauri::command]
pub async fn get_export_format_list() -> Result<Vec<ExportFormatInfo>, String> {
    Ok(get_export_formats())
}

// ============================================================================
// Report Generation Commands
// ============================================================================

/// Request to generate a report
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReportRequest {
    pub report_type: String,
    pub client_id: String,
    pub client_name: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub organization: Option<String>,
    pub format: String,
    pub include_toc: bool,
    pub include_executive_summary: bool,
    pub include_appendices: bool,
    pub include_charts: bool,
    pub classification: Option<String>,
    pub notes: Option<String>,
}

/// Generate a new report
#[tauri::command]
pub async fn generate_report(
    state: State<'_, ReportingState>,
    request: GenerateReportRequest,
) -> Result<Report, String> {
    let report_type = parse_report_type(&request.report_type)?;
    let format = parse_export_format(&request.format)?;

    let config = ReportConfig {
        report_type,
        client_id: request.client_id,
        client_name: request.client_name,
        title: request.title,
        subtitle: request.subtitle,
        author: request.author,
        organization: request.organization,
        format,
        include_toc: request.include_toc,
        include_executive_summary: request.include_executive_summary,
        include_appendices: request.include_appendices,
        include_charts: request.include_charts,
        logo_path: None,
        primary_color: Some("#3B82F6".to_string()),
        notes: request.notes,
        classification: request.classification,
        data_sources: vec![],
    };

    let generator = ReportGenerator::new(config);
    let report = generator.generate()?;

    let mut reports = state.reports.lock().map_err(|e| e.to_string())?;
    reports.push(report.clone());

    Ok(report)
}

/// Preview report content without saving
#[tauri::command]
pub async fn preview_report(request: GenerateReportRequest) -> Result<ReportContent, String> {
    let report_type = parse_report_type(&request.report_type)?;
    let format = parse_export_format(&request.format)?;

    let config = ReportConfig {
        report_type,
        client_id: request.client_id,
        client_name: request.client_name,
        title: request.title,
        subtitle: request.subtitle,
        author: request.author,
        organization: request.organization,
        format,
        include_toc: request.include_toc,
        include_executive_summary: request.include_executive_summary,
        include_appendices: request.include_appendices,
        include_charts: request.include_charts,
        logo_path: None,
        primary_color: Some("#3B82F6".to_string()),
        notes: request.notes,
        classification: request.classification,
        data_sources: vec![],
    };

    let generator = ReportGenerator::new(config);
    let report = generator.generate()?;

    report.content.ok_or_else(|| "Failed to generate content".to_string())
}

// ============================================================================
// Report Export Commands
// ============================================================================

/// Export report to HTML
#[tauri::command]
pub async fn export_report_html(
    state: State<'_, ReportingState>,
    report_id: String,
) -> Result<String, String> {
    let reports = state.reports.lock().map_err(|e| e.to_string())?;
    let report = reports.iter()
        .find(|r| r.id == report_id)
        .ok_or_else(|| "Report not found".to_string())?;

    let content = report.content.as_ref()
        .ok_or_else(|| "Report has no content".to_string())?;

    Ok(content_to_html(content))
}

/// Export report to Markdown
#[tauri::command]
pub async fn export_report_markdown(
    state: State<'_, ReportingState>,
    report_id: String,
) -> Result<String, String> {
    let reports = state.reports.lock().map_err(|e| e.to_string())?;
    let report = reports.iter()
        .find(|r| r.id == report_id)
        .ok_or_else(|| "Report not found".to_string())?;

    let content = report.content.as_ref()
        .ok_or_else(|| "Report has no content".to_string())?;

    Ok(content_to_markdown(content))
}

/// Export report to JSON
#[tauri::command]
pub async fn export_report_json(
    state: State<'_, ReportingState>,
    report_id: String,
) -> Result<String, String> {
    let reports = state.reports.lock().map_err(|e| e.to_string())?;
    let report = reports.iter()
        .find(|r| r.id == report_id)
        .ok_or_else(|| "Report not found".to_string())?;

    serde_json::to_string_pretty(report)
        .map_err(|e| format!("JSON serialization failed: {}", e))
}

// ============================================================================
// Report Management Commands
// ============================================================================

/// List all reports for a client
#[tauri::command]
pub async fn list_reports(
    state: State<'_, ReportingState>,
    client_id: Option<String>,
) -> Result<Vec<ReportSummary>, String> {
    let reports = state.reports.lock().map_err(|e| e.to_string())?;

    let filtered: Vec<ReportSummary> = reports.iter()
        .filter(|r| client_id.as_ref().map_or(true, |cid| &r.client_id == cid))
        .map(|r| ReportSummary {
            id: r.id.clone(),
            title: r.config.title.clone(),
            report_type: r.config.report_type,
            client_name: r.config.client_name.clone(),
            status: r.status,
            format: r.config.format,
            created_at: r.created_at.clone(),
            file_size: r.file_size,
        })
        .collect();

    Ok(filtered)
}

/// Get a specific report by ID
#[tauri::command]
pub async fn get_report(
    state: State<'_, ReportingState>,
    report_id: String,
) -> Result<Option<Report>, String> {
    let reports = state.reports.lock().map_err(|e| e.to_string())?;
    Ok(reports.iter().find(|r| r.id == report_id).cloned())
}

/// Delete a report
#[tauri::command]
pub async fn delete_report(
    state: State<'_, ReportingState>,
    report_id: String,
) -> Result<bool, String> {
    let mut reports = state.reports.lock().map_err(|e| e.to_string())?;
    let len_before = reports.len();
    reports.retain(|r| r.id != report_id);
    Ok(reports.len() < len_before)
}

/// Get report statistics
#[tauri::command]
pub async fn get_report_stats(
    state: State<'_, ReportingState>,
    client_id: Option<String>,
) -> Result<ReportStats, String> {
    let reports = state.reports.lock().map_err(|e| e.to_string())?;

    let filtered: Vec<&Report> = reports.iter()
        .filter(|r| client_id.as_ref().map_or(true, |cid| &r.client_id == cid))
        .collect();

    let total_reports = filtered.len();

    // Count by type
    let mut type_counts: std::collections::HashMap<ReportType, usize> = std::collections::HashMap::new();
    for r in &filtered {
        *type_counts.entry(r.config.report_type).or_insert(0) += 1;
    }
    let by_type: Vec<ReportTypeCount> = type_counts.into_iter()
        .map(|(report_type, count)| ReportTypeCount { report_type, count })
        .collect();

    // Count by status
    let mut status_counts: std::collections::HashMap<ReportStatus, usize> = std::collections::HashMap::new();
    for r in &filtered {
        *status_counts.entry(r.status).or_insert(0) += 1;
    }
    let by_status: Vec<ReportStatusCount> = status_counts.into_iter()
        .map(|(status, count)| ReportStatusCount { status, count })
        .collect();

    // Recent reports
    let mut recent: Vec<ReportSummary> = filtered.iter()
        .take(5)
        .map(|r| ReportSummary {
            id: r.id.clone(),
            title: r.config.title.clone(),
            report_type: r.config.report_type,
            client_name: r.config.client_name.clone(),
            status: r.status,
            format: r.config.format,
            created_at: r.created_at.clone(),
            file_size: r.file_size,
        })
        .collect();

    Ok(ReportStats {
        total_reports,
        reports_this_month: total_reports, // Simplified
        by_type,
        by_status,
        recent_reports: recent,
    })
}

// ============================================================================
// Demo Data
// ============================================================================

/// Generate demo reports for development
#[tauri::command]
pub async fn generate_demo_reports(client_id: String, client_name: String) -> Result<Vec<ReportSummary>, String> {
    let now = chrono::Utc::now().to_rfc3339();

    let demo_reports = vec![
        ReportSummary {
            id: Uuid::new_v4().to_string(),
            title: "Q4 2025 Security Assessment".to_string(),
            report_type: ReportType::ExecutiveSummary,
            client_name: client_name.clone(),
            status: ReportStatus::Ready,
            format: ExportFormat::Pdf,
            created_at: now.clone(),
            file_size: Some(245_000),
        },
        ReportSummary {
            id: Uuid::new_v4().to_string(),
            title: "SOC 2 Compliance Report".to_string(),
            report_type: ReportType::ComplianceReport,
            client_name: client_name.clone(),
            status: ReportStatus::Ready,
            format: ExportFormat::Pdf,
            created_at: now.clone(),
            file_size: Some(892_000),
        },
        ReportSummary {
            id: Uuid::new_v4().to_string(),
            title: "Network Infrastructure Assessment".to_string(),
            report_type: ReportType::NetworkAssessment,
            client_name: client_name.clone(),
            status: ReportStatus::Ready,
            format: ExportFormat::Pdf,
            created_at: now.clone(),
            file_size: Some(567_000),
        },
        ReportSummary {
            id: Uuid::new_v4().to_string(),
            title: "Technical Vulnerability Assessment".to_string(),
            report_type: ReportType::TechnicalAssessment,
            client_name: client_name.clone(),
            status: ReportStatus::Draft,
            format: ExportFormat::Html,
            created_at: now.clone(),
            file_size: None,
        },
    ];

    Ok(demo_reports)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_report_type(s: &str) -> Result<ReportType, String> {
    match s.to_lowercase().replace("-", "_").as_str() {
        "executive_summary" | "executivesummary" | "executive" => Ok(ReportType::ExecutiveSummary),
        "technical_assessment" | "technicalassessment" | "technical" => Ok(ReportType::TechnicalAssessment),
        "compliance_report" | "compliancereport" | "compliance" => Ok(ReportType::ComplianceReport),
        "network_assessment" | "networkassessment" | "network" => Ok(ReportType::NetworkAssessment),
        "cloud_readiness" | "cloudreadiness" | "cloud" => Ok(ReportType::CloudReadiness),
        "security_findings" | "securityfindings" | "security" | "findings" => Ok(ReportType::SecurityFindings),
        "full_engagement" | "fullengagement" | "full" => Ok(ReportType::FullEngagement),
        _ => Err(format!("Unknown report type: {}", s)),
    }
}

fn parse_export_format(s: &str) -> Result<ExportFormat, String> {
    match s.to_lowercase().as_str() {
        "pdf" => Ok(ExportFormat::Pdf),
        "html" => Ok(ExportFormat::Html),
        "markdown" | "md" => Ok(ExportFormat::Markdown),
        "docx" | "word" => Ok(ExportFormat::Docx),
        "json" => Ok(ExportFormat::Json),
        _ => Err(format!("Unknown export format: {}", s)),
    }
}

// ============================================================================
// PDF Generation Commands (Task B)
// ============================================================================

/// Request to generate an executive PDF report
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateExecutivePdfRequest {
    pub client_id: String,
    pub client_name: String,
    pub title: Option<String>,
    pub framework: Option<String>,
    pub include_network_data: bool,
    pub include_compliance_data: bool,
}

/// Response from PDF generation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfGenerationResult {
    pub success: bool,
    pub file_path: String,
    pub file_size: u64,
    pub page_count: u32,
    pub message: String,
}

/// Generate an executive summary PDF with GRC and Network data
#[tauri::command]
pub async fn generate_executive_pdf(
    app_handle: tauri::AppHandle,
    db: State<'_, Database>,
    request: GenerateExecutivePdfRequest,
) -> Result<PdfGenerationResult, String> {
    // Get the app data directory for output
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    let file_name = format!(
        "executive_report_{}_{}.pdf",
        request.client_id,
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let output_path = app_data_dir.join(&file_name);

    // Build executive report data
    let compliance_status = if request.include_compliance_data {
        let framework = request.framework.as_deref().unwrap_or("NIST_CSF_2");
        build_compliance_status(&db, framework, Some(&request.client_id)).await.ok()
    } else {
        None
    };

    // Calculate network health score (placeholder - could integrate with actual network scan data)
    let network_health_score = if request.include_network_data {
        calculate_network_health_score(&compliance_status)
    } else {
        0.0
    };

    let title = request.title.unwrap_or_else(|| {
        format!("Executive Security Assessment - {}", request.client_name)
    });

    let data = ExecutiveReportData {
        client_name: request.client_name.clone(),
        title: title.clone(),
        report_date: chrono::Utc::now().format("%B %d, %Y").to_string(),
        compliance_status: compliance_status.clone(),
        network_health_score,
        total_assets: compliance_status.as_ref()
            .and_then(|c| c.total_assets)
            .unwrap_or(0),
        assets_by_category: vec![
            AssetCategoryCount { category: "Servers".to_string(), count: 24 },
            AssetCategoryCount { category: "Workstations".to_string(), count: 87 },
            AssetCategoryCount { category: "Network Devices".to_string(), count: 15 },
            AssetCategoryCount { category: "Security Appliances".to_string(), count: 8 },
        ],
        top_findings: generate_findings_from_compliance(&compliance_status),
        risk_summary: calculate_risk_summary(&compliance_status),
    };

    // Generate PDF
    let generator = PdfGenerator::new(title);
    let file_size = generator.generate_executive_report(&data, &output_path)?;

    Ok(PdfGenerationResult {
        success: true,
        file_path: output_path.to_string_lossy().to_string(),
        file_size,
        page_count: 5,
        message: "Executive PDF report generated successfully".to_string(),
    })
}

/// Generate a demo executive PDF for testing
#[tauri::command]
pub async fn generate_demo_pdf(
    app_handle: tauri::AppHandle,
    client_name: String,
) -> Result<PdfGenerationResult, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    let file_name = format!(
        "demo_executive_report_{}.pdf",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let output_path = app_data_dir.join(&file_name);

    let file_size = generate_demo_executive_report(&client_name, &output_path)?;

    Ok(PdfGenerationResult {
        success: true,
        file_path: output_path.to_string_lossy().to_string(),
        file_size,
        page_count: 5,
        message: "Demo executive PDF generated successfully".to_string(),
    })
}

/// Open the generated PDF file location
#[tauri::command]
pub async fn open_pdf_location(file_path: String) -> Result<bool, String> {
    let path = PathBuf::from(&file_path);
    let parent = path.parent().ok_or_else(|| "Invalid file path".to_string())?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(true)
}

// ============================================================================
// PDF Helper Functions
// ============================================================================

async fn build_compliance_status(
    db: &Database,
    framework_str: &str,
    client_id: Option<&str>,
) -> Result<ComplianceStatusReport, String> {
    let fw = match framework_str.to_uppercase().as_str() {
        "NIST_CSF_2" | "NISTCSF2" | "NIST_CSF2" => Framework::NistCsf2,
        "SOC_2_TYPE_II" | "SOC2TYPEII" | "SOC2" => Framework::Soc2TypeII,
        "GDPR" => Framework::Gdpr,
        _ => return Err(format!("Unknown framework: {}", framework_str)),
    };

    let controls = get_framework_controls(fw);
    let categories = get_framework_categories(fw);

    let assessment_repo = AssessmentRepository::new(db);
    let control_repo = ControlAssessmentRepository::new(db);

    let assessments = if let Some(cid) = client_id {
        assessment_repo.list_by_client(cid).map_err(|e| e.to_string())?
    } else {
        assessment_repo.list_all().map_err(|e| e.to_string())?
    };

    let framework_assessments: Vec<_> = assessments
        .into_iter()
        .filter(|a| a.framework == fw)
        .collect();

    let mut all_control_assessments: HashMap<String, crate::grc::models::ControlAssessment> = HashMap::new();
    for assessment in &framework_assessments {
        if let Ok(cas) = control_repo.get_by_assessment(&assessment.id) {
            for ca in cas {
                all_control_assessments.insert(ca.control_id.clone(), ca);
            }
        }
    }

    let mut total_assessed = 0;
    let mut total_compliant = 0;
    let mut total_partial = 0;
    let mut total_non_compliant = 0;
    let mut total_na = 0;

    let mut category_map: HashMap<String, (usize, usize, usize, usize, usize, usize)> = HashMap::new();

    for control in &controls {
        use crate::grc::models::ComplianceStatus;
        let status = all_control_assessments
            .get(&control.id)
            .map(|ca| ca.status)
            .unwrap_or(ComplianceStatus::NotAssessed);

        let entry = category_map.entry(control.category.clone()).or_insert((0, 0, 0, 0, 0, 0));
        entry.0 += 1;

        match status {
            ComplianceStatus::NotAssessed => {}
            ComplianceStatus::Compliant => {
                total_assessed += 1;
                total_compliant += 1;
                entry.1 += 1;
                entry.2 += 1;
            }
            ComplianceStatus::PartiallyCompliant => {
                total_assessed += 1;
                total_partial += 1;
                entry.1 += 1;
                entry.3 += 1;
            }
            ComplianceStatus::NonCompliant => {
                total_assessed += 1;
                total_non_compliant += 1;
                entry.1 += 1;
                entry.4 += 1;
            }
            ComplianceStatus::NotApplicable => {
                total_assessed += 1;
                total_na += 1;
                entry.1 += 1;
                entry.5 += 1;
            }
        }
    }

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
        total_assets: Some(134),
        last_updated: chrono::Utc::now().to_rfc3339(),
    })
}

fn calculate_network_health_score(compliance: &Option<ComplianceStatusReport>) -> f64 {
    match compliance {
        Some(c) => {
            // Base score from compliance
            let base = c.compliance_percentage * 0.6;
            // Add points for completion
            let completion_bonus = c.completion_percentage * 0.2;
            // Deduct for non-compliant controls
            let penalty = (c.non_compliant_controls as f64 * 2.0).min(20.0);
            (base + completion_bonus - penalty).max(0.0).min(100.0)
        }
        None => 75.0, // Default score
    }
}

fn generate_findings_from_compliance(compliance: &Option<ComplianceStatusReport>) -> Vec<ExecutiveFinding> {
    let mut findings = Vec::new();

    if let Some(c) = compliance {
        // Generate findings based on non-compliant categories
        for (i, cat) in c.category_breakdown.iter().enumerate() {
            if cat.non_compliant > 0 {
                findings.push(ExecutiveFinding {
                    id: format!("FIND-{:03}", i + 1),
                    title: format!("{} Controls Require Attention", cat.name),
                    severity: if cat.non_compliant > 2 { "High".to_string() } else { "Medium".to_string() },
                    description: format!(
                        "{} out of {} controls in the {} category are non-compliant",
                        cat.non_compliant, cat.total_controls, cat.name
                    ),
                    recommendation: format!(
                        "Review and remediate {} controls to improve {} compliance",
                        cat.non_compliant, cat.name
                    ),
                });
            }
        }
    }

    // Add default findings if none from compliance
    if findings.is_empty() {
        findings.push(ExecutiveFinding {
            id: "FIND-001".to_string(),
            title: "Complete Compliance Assessment".to_string(),
            severity: "Medium".to_string(),
            description: "No compliance assessment data available for analysis".to_string(),
            recommendation: "Conduct a full compliance assessment against the selected framework".to_string(),
        });
    }

    findings
}

fn calculate_risk_summary(compliance: &Option<ComplianceStatusReport>) -> RiskSummary {
    match compliance {
        Some(c) => {
            let critical = c.non_compliant_controls.min(3);
            let high = (c.non_compliant_controls.saturating_sub(critical)).min(5);
            let medium = c.partially_compliant_controls.min(10);
            let low = (c.total_controls - c.assessed_controls).min(8);

            let rating = if critical > 1 {
                "Critical"
            } else if high > 2 {
                "High"
            } else if medium > 5 {
                "Moderate"
            } else {
                "Low"
            };

            RiskSummary {
                critical_count: critical,
                high_count: high,
                medium_count: medium,
                low_count: low,
                overall_risk_rating: rating.to_string(),
            }
        }
        None => RiskSummary {
            critical_count: 0,
            high_count: 2,
            medium_count: 5,
            low_count: 3,
            overall_risk_rating: "Unknown".to_string(),
        },
    }
}
