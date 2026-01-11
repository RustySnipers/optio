//! Reporting Commands
//!
//! Tauri commands for report generation and management.

use crate::reporting::{
    models::*,
    generator::{ReportGenerator, content_to_html, content_to_markdown},
    templates::{get_report_templates, get_template_for_type, get_report_type_info, get_export_formats, ReportTypeInfo, ExportFormatInfo},
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
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
