//! Reporting Data Models
//!
//! Types for report generation, templates, and export formats.

use serde::{Deserialize, Serialize};

// ============================================================================
// Report Types
// ============================================================================

/// Type of report to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    /// High-level summary for executives and stakeholders
    ExecutiveSummary,
    /// Detailed technical assessment report
    TechnicalAssessment,
    /// Compliance status and gap analysis
    ComplianceReport,
    /// Network discovery and asset inventory
    NetworkAssessment,
    /// Cloud migration readiness report
    CloudReadiness,
    /// Security findings and recommendations
    SecurityFindings,
    /// Full engagement report combining all modules
    FullEngagement,
}

impl ReportType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ReportType::ExecutiveSummary => "Executive Summary",
            ReportType::TechnicalAssessment => "Technical Assessment",
            ReportType::ComplianceReport => "Compliance Report",
            ReportType::NetworkAssessment => "Network Assessment",
            ReportType::CloudReadiness => "Cloud Readiness Report",
            ReportType::SecurityFindings => "Security Findings",
            ReportType::FullEngagement => "Full Engagement Report",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ReportType::ExecutiveSummary => "High-level overview for executives with key findings, risk summary, and strategic recommendations",
            ReportType::TechnicalAssessment => "Detailed technical findings, vulnerability analysis, and remediation guidance",
            ReportType::ComplianceReport => "Framework compliance status, control assessments, and gap analysis",
            ReportType::NetworkAssessment => "Network topology, asset inventory, and infrastructure analysis",
            ReportType::CloudReadiness => "Cloud migration readiness assessment with cost projections",
            ReportType::SecurityFindings => "Security vulnerabilities, risk ratings, and prioritized remediation",
            ReportType::FullEngagement => "Comprehensive report combining all assessment modules",
        }
    }

    pub fn estimated_pages(&self) -> &'static str {
        match self {
            ReportType::ExecutiveSummary => "5-10 pages",
            ReportType::TechnicalAssessment => "20-50 pages",
            ReportType::ComplianceReport => "15-40 pages",
            ReportType::NetworkAssessment => "10-30 pages",
            ReportType::CloudReadiness => "15-25 pages",
            ReportType::SecurityFindings => "10-30 pages",
            ReportType::FullEngagement => "50-100+ pages",
        }
    }
}

/// Export format for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Pdf,
    Html,
    Markdown,
    Docx,
    Json,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Html => "html",
            ExportFormat::Markdown => "md",
            ExportFormat::Docx => "docx",
            ExportFormat::Json => "json",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::Html => "text/html",
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ExportFormat::Json => "application/json",
        }
    }
}

/// Report status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Draft,
    Generating,
    Ready,
    Failed,
    Archived,
}

// ============================================================================
// Report Configuration
// ============================================================================

/// Configuration for generating a report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportConfig {
    /// Type of report to generate
    pub report_type: ReportType,
    /// Client information
    pub client_id: String,
    pub client_name: String,
    /// Report title
    pub title: String,
    /// Report subtitle/description
    pub subtitle: Option<String>,
    /// Consultant/author name
    pub author: String,
    /// Company/organization name
    pub organization: Option<String>,
    /// Export format
    pub format: ExportFormat,
    /// Include table of contents
    pub include_toc: bool,
    /// Include executive summary section
    pub include_executive_summary: bool,
    /// Include appendices
    pub include_appendices: bool,
    /// Include charts and visualizations
    pub include_charts: bool,
    /// Branding/logo path
    pub logo_path: Option<String>,
    /// Primary brand color
    pub primary_color: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// Classification level
    pub classification: Option<String>,
    /// Data sources to include
    pub data_sources: Vec<DataSource>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            report_type: ReportType::ExecutiveSummary,
            client_id: String::new(),
            client_name: String::new(),
            title: "Security Assessment Report".to_string(),
            subtitle: None,
            author: String::new(),
            organization: None,
            format: ExportFormat::Pdf,
            include_toc: true,
            include_executive_summary: true,
            include_appendices: true,
            include_charts: true,
            logo_path: None,
            primary_color: Some("#3B82F6".to_string()),
            notes: None,
            classification: Some("Confidential".to_string()),
            data_sources: vec![],
        }
    }
}

/// Data source to include in report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSource {
    /// Source type (assessment, scan, etc.)
    pub source_type: String,
    /// Source ID
    pub source_id: String,
    /// Whether to include this source
    pub included: bool,
}

// ============================================================================
// Report Structure
// ============================================================================

/// A generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Unique report identifier
    pub id: String,
    /// Client ID
    pub client_id: String,
    /// Report configuration used
    pub config: ReportConfig,
    /// Current status
    pub status: ReportStatus,
    /// Generated content (for preview)
    pub content: Option<ReportContent>,
    /// File path if exported
    pub file_path: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// When report was created
    pub created_at: String,
    /// When report was last updated
    pub updated_at: String,
    /// Generation error message
    pub error: Option<String>,
}

/// Report content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportContent {
    /// Report sections
    pub sections: Vec<ReportSection>,
    /// Report metadata
    pub metadata: ReportMetadata,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub organization: Option<String>,
    pub client_name: String,
    pub report_date: String,
    pub classification: Option<String>,
    pub version: String,
    pub page_count: Option<u32>,
}

/// A section within a report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSection {
    /// Section ID
    pub id: String,
    /// Section title
    pub title: String,
    /// Section level (1 = H1, 2 = H2, etc.)
    pub level: u8,
    /// Section content blocks
    pub blocks: Vec<ContentBlock>,
    /// Child sections
    pub subsections: Vec<ReportSection>,
}

/// Content block within a section
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "paragraph")]
    Paragraph { text: String },

    #[serde(rename = "heading")]
    Heading { text: String, level: u8 },

    #[serde(rename = "bullet_list")]
    BulletList { items: Vec<String> },

    #[serde(rename = "numbered_list")]
    NumberedList { items: Vec<String> },

    #[serde(rename = "table")]
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        caption: Option<String>,
    },

    #[serde(rename = "chart")]
    Chart {
        chart_type: ChartType,
        title: String,
        data: ChartData,
    },

    #[serde(rename = "key_value")]
    KeyValue { items: Vec<KeyValueItem> },

    #[serde(rename = "callout")]
    Callout {
        callout_type: CalloutType,
        title: Option<String>,
        text: String,
    },

    #[serde(rename = "code")]
    Code { language: Option<String>, content: String },

    #[serde(rename = "finding")]
    Finding {
        id: String,
        title: String,
        severity: String,
        description: String,
        impact: String,
        recommendation: String,
    },

    #[serde(rename = "metric")]
    Metric {
        label: String,
        value: String,
        change: Option<String>,
        trend: Option<String>,
    },

    #[serde(rename = "page_break")]
    PageBreak,
}

/// Chart type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Bar,
    Pie,
    Line,
    Donut,
    Radar,
    Heatmap,
    Gauge,
}

/// Chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartData {
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

/// Chart dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: Option<String>,
}

/// Key-value item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValueItem {
    pub key: String,
    pub value: String,
}

/// Callout type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalloutType {
    Info,
    Warning,
    Critical,
    Success,
    Note,
}

// ============================================================================
// Report Templates
// ============================================================================

/// Report template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// Report type this template is for
    pub report_type: ReportType,
    /// Section definitions
    pub sections: Vec<TemplateSectionDef>,
    /// Default configuration
    pub default_config: ReportConfig,
}

/// Template section definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSectionDef {
    pub id: String,
    pub title: String,
    pub description: String,
    pub required: bool,
    pub default_included: bool,
}

// ============================================================================
// Report History
// ============================================================================

/// Summary of a generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSummary {
    pub id: String,
    pub title: String,
    pub report_type: ReportType,
    pub client_name: String,
    pub status: ReportStatus,
    pub format: ExportFormat,
    pub created_at: String,
    pub file_size: Option<u64>,
}

/// Statistics about reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportStats {
    pub total_reports: usize,
    pub reports_this_month: usize,
    pub by_type: Vec<ReportTypeCount>,
    pub by_status: Vec<ReportStatusCount>,
    pub recent_reports: Vec<ReportSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportTypeCount {
    pub report_type: ReportType,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportStatusCount {
    pub status: ReportStatus,
    pub count: usize,
}
