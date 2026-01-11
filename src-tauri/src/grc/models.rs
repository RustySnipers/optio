//! GRC Data Models
//!
//! Core data structures for compliance frameworks, controls, evidence,
//! and assessments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported compliance frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Framework {
    /// NIST Cybersecurity Framework 2.0
    NistCsf2,
    /// SOC 2 Type II
    Soc2TypeII,
    /// General Data Protection Regulation
    Gdpr,
}

impl Framework {
    pub fn display_name(&self) -> &'static str {
        match self {
            Framework::NistCsf2 => "NIST CSF 2.0",
            Framework::Soc2TypeII => "SOC 2 Type II",
            Framework::Gdpr => "GDPR",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Framework::NistCsf2 => "NIST Cybersecurity Framework version 2.0 - A voluntary framework for managing cybersecurity risk",
            Framework::Soc2TypeII => "Service Organization Control 2 Type II - Trust Services Criteria for security, availability, processing integrity, confidentiality, and privacy",
            Framework::Gdpr => "General Data Protection Regulation - EU regulation on data protection and privacy",
        }
    }

    pub fn all() -> Vec<Framework> {
        vec![Framework::NistCsf2, Framework::Soc2TypeII, Framework::Gdpr]
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// NIST CSF 2.0 Functions (top-level categories)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NistFunction {
    Govern,
    Identify,
    Protect,
    Detect,
    Respond,
    Recover,
}

impl NistFunction {
    pub fn code(&self) -> &'static str {
        match self {
            NistFunction::Govern => "GV",
            NistFunction::Identify => "ID",
            NistFunction::Protect => "PR",
            NistFunction::Detect => "DE",
            NistFunction::Respond => "RS",
            NistFunction::Recover => "RC",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            NistFunction::Govern => "Govern",
            NistFunction::Identify => "Identify",
            NistFunction::Protect => "Protect",
            NistFunction::Detect => "Detect",
            NistFunction::Respond => "Respond",
            NistFunction::Recover => "Recover",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NistFunction::Govern => "Establish and monitor the organization's cybersecurity risk management strategy, expectations, and policy",
            NistFunction::Identify => "Understand the organization's current cybersecurity risk posture",
            NistFunction::Protect => "Use safeguards to manage cybersecurity risks",
            NistFunction::Detect => "Find and analyze possible cybersecurity attacks and compromises",
            NistFunction::Respond => "Take action regarding a detected cybersecurity incident",
            NistFunction::Recover => "Restore assets and operations affected by a cybersecurity incident",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            NistFunction::Govern => "#8b5cf6",    // Purple
            NistFunction::Identify => "#3b82f6",  // Blue
            NistFunction::Protect => "#22c55e",   // Green
            NistFunction::Detect => "#f59e0b",    // Amber
            NistFunction::Respond => "#ef4444",   // Red
            NistFunction::Recover => "#06b6d4",   // Cyan
        }
    }

    pub fn all() -> Vec<NistFunction> {
        vec![
            NistFunction::Govern,
            NistFunction::Identify,
            NistFunction::Protect,
            NistFunction::Detect,
            NistFunction::Respond,
            NistFunction::Recover,
        ]
    }
}

/// SOC 2 Trust Services Categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Soc2Category {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

impl Soc2Category {
    pub fn code(&self) -> &'static str {
        match self {
            Soc2Category::Security => "CC",
            Soc2Category::Availability => "A",
            Soc2Category::ProcessingIntegrity => "PI",
            Soc2Category::Confidentiality => "C",
            Soc2Category::Privacy => "P",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Soc2Category::Security => "Security",
            Soc2Category::Availability => "Availability",
            Soc2Category::ProcessingIntegrity => "Processing Integrity",
            Soc2Category::Confidentiality => "Confidentiality",
            Soc2Category::Privacy => "Privacy",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Soc2Category::Security => "#3b82f6",
            Soc2Category::Availability => "#22c55e",
            Soc2Category::ProcessingIntegrity => "#f59e0b",
            Soc2Category::Confidentiality => "#8b5cf6",
            Soc2Category::Privacy => "#ec4899",
        }
    }

    pub fn all() -> Vec<Soc2Category> {
        vec![
            Soc2Category::Security,
            Soc2Category::Availability,
            Soc2Category::ProcessingIntegrity,
            Soc2Category::Confidentiality,
            Soc2Category::Privacy,
        ]
    }
}

/// GDPR Articles/Chapters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GdprChapter {
    Principles,
    DataSubjectRights,
    ControllerProcessor,
    TransferToThirdCountries,
    SupervisoryAuthorities,
    Remedies,
}

impl GdprChapter {
    pub fn code(&self) -> &'static str {
        match self {
            GdprChapter::Principles => "CH2",
            GdprChapter::DataSubjectRights => "CH3",
            GdprChapter::ControllerProcessor => "CH4",
            GdprChapter::TransferToThirdCountries => "CH5",
            GdprChapter::SupervisoryAuthorities => "CH6",
            GdprChapter::Remedies => "CH8",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            GdprChapter::Principles => "Principles",
            GdprChapter::DataSubjectRights => "Data Subject Rights",
            GdprChapter::ControllerProcessor => "Controller & Processor",
            GdprChapter::TransferToThirdCountries => "Transfers to Third Countries",
            GdprChapter::SupervisoryAuthorities => "Supervisory Authorities",
            GdprChapter::Remedies => "Remedies & Penalties",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            GdprChapter::Principles => "#3b82f6",
            GdprChapter::DataSubjectRights => "#22c55e",
            GdprChapter::ControllerProcessor => "#f59e0b",
            GdprChapter::TransferToThirdCountries => "#8b5cf6",
            GdprChapter::SupervisoryAuthorities => "#ec4899",
            GdprChapter::Remedies => "#ef4444",
        }
    }

    pub fn all() -> Vec<GdprChapter> {
        vec![
            GdprChapter::Principles,
            GdprChapter::DataSubjectRights,
            GdprChapter::ControllerProcessor,
            GdprChapter::TransferToThirdCountries,
            GdprChapter::SupervisoryAuthorities,
            GdprChapter::Remedies,
        ]
    }
}

/// A compliance control/requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Control {
    /// Unique identifier
    pub id: String,
    /// Framework this control belongs to
    pub framework: Framework,
    /// Control code (e.g., "PR.AC-1", "CC6.1", "Art. 25")
    pub code: String,
    /// Category/Function/Chapter this control belongs to
    pub category: String,
    /// Subcategory if applicable
    pub subcategory: Option<String>,
    /// Control title
    pub title: String,
    /// Full description of the control
    pub description: String,
    /// Implementation guidance
    pub guidance: Option<String>,
    /// Related controls in other frameworks
    pub cross_references: Vec<String>,
    /// Priority/importance level (1-5)
    pub priority: u8,
}

/// Compliance status for a control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceStatus {
    /// Not yet assessed
    NotAssessed,
    /// Fully compliant
    Compliant,
    /// Partially compliant (some gaps)
    PartiallyCompliant,
    /// Not compliant
    NonCompliant,
    /// Not applicable to this organization
    NotApplicable,
}

impl ComplianceStatus {
    pub fn score(&self) -> f64 {
        match self {
            ComplianceStatus::NotAssessed => 0.0,
            ComplianceStatus::Compliant => 1.0,
            ComplianceStatus::PartiallyCompliant => 0.5,
            ComplianceStatus::NonCompliant => 0.0,
            ComplianceStatus::NotApplicable => 1.0, // N/A counts as compliant for scoring
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ComplianceStatus::NotAssessed => "Not Assessed",
            ComplianceStatus::Compliant => "Compliant",
            ComplianceStatus::PartiallyCompliant => "Partially Compliant",
            ComplianceStatus::NonCompliant => "Non-Compliant",
            ComplianceStatus::NotApplicable => "Not Applicable",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ComplianceStatus::NotAssessed => "#64748b",
            ComplianceStatus::Compliant => "#22c55e",
            ComplianceStatus::PartiallyCompliant => "#f59e0b",
            ComplianceStatus::NonCompliant => "#ef4444",
            ComplianceStatus::NotApplicable => "#94a3b8",
        }
    }
}

/// Evidence type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceType {
    /// Document (policy, procedure, etc.)
    Document,
    /// Screenshot or image
    Screenshot,
    /// Configuration export
    Configuration,
    /// Scan/audit result
    ScanResult,
    /// Interview notes
    Interview,
    /// Log file or excerpt
    LogFile,
    /// Other evidence
    Other,
}

impl EvidenceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            EvidenceType::Document => "Document",
            EvidenceType::Screenshot => "Screenshot",
            EvidenceType::Configuration => "Configuration",
            EvidenceType::ScanResult => "Scan Result",
            EvidenceType::Interview => "Interview",
            EvidenceType::LogFile => "Log File",
            EvidenceType::Other => "Other",
        }
    }
}

/// Evidence collected for a control
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    /// Unique identifier
    pub id: String,
    /// Assessment this evidence belongs to
    pub assessment_id: String,
    /// Control(s) this evidence supports
    pub control_ids: Vec<String>,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Title/name
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// File path (if stored locally)
    pub file_path: Option<String>,
    /// External URL (if applicable)
    pub url: Option<String>,
    /// Hash of the evidence file for integrity
    pub file_hash: Option<String>,
    /// When the evidence was collected
    pub collected_at: DateTime<Utc>,
    /// Who collected/uploaded the evidence
    pub collected_by: String,
    /// Notes about the evidence
    pub notes: Option<String>,
}

/// Assessment of a control within an engagement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlAssessment {
    /// Unique identifier
    pub id: String,
    /// Assessment this belongs to
    pub assessment_id: String,
    /// Control being assessed
    pub control_id: String,
    /// Current compliance status
    pub status: ComplianceStatus,
    /// Assessor notes
    pub notes: Option<String>,
    /// Gap description if not fully compliant
    pub gap_description: Option<String>,
    /// Remediation recommendation
    pub remediation: Option<String>,
    /// Target date for remediation
    pub remediation_target: Option<DateTime<Utc>>,
    /// Risk rating if non-compliant (1-5)
    pub risk_rating: Option<u8>,
    /// Evidence IDs supporting this assessment
    pub evidence_ids: Vec<String>,
    /// When last assessed
    pub assessed_at: DateTime<Utc>,
    /// Who performed the assessment
    pub assessed_by: String,
}

/// A compliance assessment/audit engagement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assessment {
    /// Unique identifier
    pub id: String,
    /// Client this assessment is for
    pub client_id: String,
    /// Assessment name/title
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Framework being assessed
    pub framework: Framework,
    /// Assessment scope
    pub scope: Option<String>,
    /// When the assessment started
    pub started_at: DateTime<Utc>,
    /// When the assessment was completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Lead assessor
    pub lead_assessor: String,
    /// Assessment status
    pub status: AssessmentStatus,
}

/// Status of an assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssessmentStatus {
    /// Not yet started
    Draft,
    /// Currently in progress
    InProgress,
    /// Under review
    UnderReview,
    /// Completed
    Completed,
    /// Archived
    Archived,
}

impl AssessmentStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            AssessmentStatus::Draft => "Draft",
            AssessmentStatus::InProgress => "In Progress",
            AssessmentStatus::UnderReview => "Under Review",
            AssessmentStatus::Completed => "Completed",
            AssessmentStatus::Archived => "Archived",
        }
    }
}

/// Summary statistics for a category/function
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryScore {
    /// Category identifier
    pub category: String,
    /// Display name
    pub display_name: String,
    /// Color for visualization
    pub color: String,
    /// Total controls in this category
    pub total_controls: usize,
    /// Compliant controls
    pub compliant: usize,
    /// Partially compliant controls
    pub partially_compliant: usize,
    /// Non-compliant controls
    pub non_compliant: usize,
    /// Not assessed controls
    pub not_assessed: usize,
    /// Not applicable controls
    pub not_applicable: usize,
    /// Compliance percentage (0-100)
    pub compliance_percentage: f64,
}

/// Overall assessment summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentSummary {
    /// Assessment ID
    pub assessment_id: String,
    /// Framework
    pub framework: Framework,
    /// Overall compliance percentage
    pub overall_compliance: f64,
    /// Total controls assessed
    pub total_controls: usize,
    /// Controls by status
    pub compliant: usize,
    pub partially_compliant: usize,
    pub non_compliant: usize,
    pub not_assessed: usize,
    pub not_applicable: usize,
    /// Breakdown by category
    pub category_scores: Vec<CategoryScore>,
    /// High-risk gaps (non-compliant with risk >= 4)
    pub high_risk_gaps: usize,
    /// Evidence count
    pub evidence_count: usize,
}

/// Compliance status for a framework (aggregate across all client assessments)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceStatusReport {
    /// Framework being reported on
    pub framework: Framework,
    /// Overall completion percentage (how many controls have been assessed)
    pub completion_percentage: f64,
    /// Overall compliance percentage (of assessed controls)
    pub compliance_percentage: f64,
    /// Total controls in the framework
    pub total_controls: usize,
    /// Controls that have been assessed
    pub assessed_controls: usize,
    /// Compliant controls
    pub compliant_controls: usize,
    /// Partially compliant controls
    pub partially_compliant_controls: usize,
    /// Non-compliant controls
    pub non_compliant_controls: usize,
    /// Not applicable controls
    pub not_applicable_controls: usize,
    /// Breakdown by category (NIST functions, SOC2 categories, etc.)
    pub category_breakdown: Vec<CategoryComplianceStatus>,
    /// Network health score (if available)
    pub network_health_score: Option<f64>,
    /// Total assets discovered
    pub total_assets: Option<usize>,
    /// Last updated timestamp
    pub last_updated: String,
}

/// Compliance status for a single category/function
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryComplianceStatus {
    /// Category code (GV, ID, PR, DE, RS, RC for NIST)
    pub code: String,
    /// Category display name
    pub name: String,
    /// Description of the category
    pub description: String,
    /// Color for visualization
    pub color: String,
    /// Total controls in this category
    pub total_controls: usize,
    /// Assessed controls in this category
    pub assessed_controls: usize,
    /// Compliant controls
    pub compliant: usize,
    /// Partially compliant
    pub partially_compliant: usize,
    /// Non-compliant
    pub non_compliant: usize,
    /// Completion percentage
    pub completion_percentage: f64,
    /// Compliance percentage
    pub compliance_percentage: f64,
}

/// Data for generating an executive report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutiveReportData {
    /// Client name
    pub client_name: String,
    /// Report title
    pub title: String,
    /// Report date
    pub report_date: String,
    /// Compliance status from GRC
    pub compliance_status: Option<ComplianceStatusReport>,
    /// Network health score (0-100)
    pub network_health_score: f64,
    /// Total assets discovered
    pub total_assets: usize,
    /// Assets by category
    pub assets_by_category: Vec<AssetCategoryCount>,
    /// Top findings/recommendations
    pub top_findings: Vec<ExecutiveFinding>,
    /// Risk summary
    pub risk_summary: RiskSummary,
}

/// Asset count by category for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetCategoryCount {
    pub category: String,
    pub count: usize,
}

/// A finding for executive reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutiveFinding {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
}

/// Risk summary for executive reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskSummary {
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub overall_risk_rating: String,
}
