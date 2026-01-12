//! Infrastructure Module Data Models
//!
//! Core data structures for cloud migration, K8s hardening, and cost analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Cloud Readiness Assessment Models
// ============================================================================

/// Cloud provider options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloudProvider {
    Aws,
    Azure,
    Gcp,
    DigitalOcean,
    OnPremises,
}

impl CloudProvider {
    pub fn display_name(&self) -> &'static str {
        match self {
            CloudProvider::Aws => "Amazon Web Services",
            CloudProvider::Azure => "Microsoft Azure",
            CloudProvider::Gcp => "Google Cloud Platform",
            CloudProvider::DigitalOcean => "DigitalOcean",
            CloudProvider::OnPremises => "On-Premises",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            CloudProvider::Aws => "AWS",
            CloudProvider::Azure => "Azure",
            CloudProvider::Gcp => "GCP",
            CloudProvider::DigitalOcean => "DO",
            CloudProvider::OnPremises => "On-Prem",
        }
    }
}

/// Migration strategy types (6 Rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MigrationStrategy {
    Rehost,      // Lift and shift
    Replatform,  // Lift, tinker, and shift
    Repurchase,  // Move to SaaS
    Refactor,    // Re-architect
    Retire,      // Decommission
    Retain,      // Keep on-premises
}

impl MigrationStrategy {
    pub fn display_name(&self) -> &'static str {
        match self {
            MigrationStrategy::Rehost => "Rehost (Lift & Shift)",
            MigrationStrategy::Replatform => "Replatform (Lift, Tinker & Shift)",
            MigrationStrategy::Repurchase => "Repurchase (Move to SaaS)",
            MigrationStrategy::Refactor => "Refactor (Re-architect)",
            MigrationStrategy::Retire => "Retire (Decommission)",
            MigrationStrategy::Retain => "Retain (Keep On-Premises)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MigrationStrategy::Rehost => "Move applications to cloud without changes",
            MigrationStrategy::Replatform => "Make minor optimizations during migration",
            MigrationStrategy::Repurchase => "Replace with cloud-native SaaS solution",
            MigrationStrategy::Refactor => "Redesign application for cloud-native architecture",
            MigrationStrategy::Retire => "Decommission applications no longer needed",
            MigrationStrategy::Retain => "Keep applications on-premises for now",
        }
    }
}

/// Cloud readiness checklist category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReadinessCategory {
    BusinessAlignment,
    TechnicalReadiness,
    SecurityCompliance,
    OperationalReadiness,
    FinancialPlanning,
    PeopleProcess,
    DataManagement,
}

impl ReadinessCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            ReadinessCategory::BusinessAlignment => "Business Alignment",
            ReadinessCategory::TechnicalReadiness => "Technical Readiness",
            ReadinessCategory::SecurityCompliance => "Security & Compliance",
            ReadinessCategory::OperationalReadiness => "Operational Readiness",
            ReadinessCategory::FinancialPlanning => "Financial Planning",
            ReadinessCategory::PeopleProcess => "People & Process",
            ReadinessCategory::DataManagement => "Data Management",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ReadinessCategory::BusinessAlignment => "#3b82f6",
            ReadinessCategory::TechnicalReadiness => "#8b5cf6",
            ReadinessCategory::SecurityCompliance => "#ef4444",
            ReadinessCategory::OperationalReadiness => "#f59e0b",
            ReadinessCategory::FinancialPlanning => "#22c55e",
            ReadinessCategory::PeopleProcess => "#ec4899",
            ReadinessCategory::DataManagement => "#06b6d4",
        }
    }

    pub fn all() -> Vec<ReadinessCategory> {
        vec![
            ReadinessCategory::BusinessAlignment,
            ReadinessCategory::TechnicalReadiness,
            ReadinessCategory::SecurityCompliance,
            ReadinessCategory::OperationalReadiness,
            ReadinessCategory::FinancialPlanning,
            ReadinessCategory::PeopleProcess,
            ReadinessCategory::DataManagement,
        ]
    }
}

/// Readiness check item status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
    NotApplicable,
}

impl CheckStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            CheckStatus::NotStarted => "Not Started",
            CheckStatus::InProgress => "In Progress",
            CheckStatus::Completed => "Completed",
            CheckStatus::Blocked => "Blocked",
            CheckStatus::NotApplicable => "Not Applicable",
        }
    }
}

/// A cloud readiness checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessCheckItem {
    pub id: String,
    pub category: ReadinessCategory,
    pub title: String,
    pub description: String,
    pub guidance: Option<String>,
    pub priority: u8,
    pub order: u32,
}

/// Assessment of a readiness check item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessCheckAssessment {
    pub id: String,
    pub assessment_id: String,
    pub check_id: String,
    pub status: CheckStatus,
    pub notes: Option<String>,
    pub blockers: Option<String>,
    pub assessed_at: DateTime<Utc>,
    pub assessed_by: String,
}

/// Cloud readiness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudReadinessAssessment {
    pub id: String,
    pub client_id: String,
    pub name: String,
    pub target_provider: CloudProvider,
    pub target_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Summary of readiness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessSummary {
    pub assessment_id: String,
    pub overall_percentage: f64,
    pub category_scores: Vec<CategoryReadinessScore>,
    pub total_checks: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub not_started: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryReadinessScore {
    pub category: ReadinessCategory,
    pub display_name: String,
    pub color: String,
    pub total: usize,
    pub completed: usize,
    pub percentage: f64,
}

// ============================================================================
// Kubernetes Hardening Models
// ============================================================================

/// K8s hardening check category (based on NSA/CISA guidelines)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum K8sHardeningCategory {
    PodSecurity,
    NetworkPolicies,
    Authentication,
    Authorization,
    Logging,
    ThreatDetection,
    SupplyChain,
    Secrets,
}

impl K8sHardeningCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            K8sHardeningCategory::PodSecurity => "Pod Security",
            K8sHardeningCategory::NetworkPolicies => "Network Policies",
            K8sHardeningCategory::Authentication => "Authentication",
            K8sHardeningCategory::Authorization => "Authorization (RBAC)",
            K8sHardeningCategory::Logging => "Logging & Monitoring",
            K8sHardeningCategory::ThreatDetection => "Threat Detection",
            K8sHardeningCategory::SupplyChain => "Supply Chain Security",
            K8sHardeningCategory::Secrets => "Secrets Management",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            K8sHardeningCategory::PodSecurity => "#ef4444",
            K8sHardeningCategory::NetworkPolicies => "#f59e0b",
            K8sHardeningCategory::Authentication => "#3b82f6",
            K8sHardeningCategory::Authorization => "#8b5cf6",
            K8sHardeningCategory::Logging => "#22c55e",
            K8sHardeningCategory::ThreatDetection => "#ec4899",
            K8sHardeningCategory::SupplyChain => "#06b6d4",
            K8sHardeningCategory::Secrets => "#f97316",
        }
    }

    pub fn all() -> Vec<K8sHardeningCategory> {
        vec![
            K8sHardeningCategory::PodSecurity,
            K8sHardeningCategory::NetworkPolicies,
            K8sHardeningCategory::Authentication,
            K8sHardeningCategory::Authorization,
            K8sHardeningCategory::Logging,
            K8sHardeningCategory::ThreatDetection,
            K8sHardeningCategory::SupplyChain,
            K8sHardeningCategory::Secrets,
        }
    }
}

/// Severity level for K8s findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn display_name(&self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Medium => "Medium",
            Severity::Low => "Low",
            Severity::Info => "Informational",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Severity::Critical => "#dc2626",
            Severity::High => "#ef4444",
            Severity::Medium => "#f59e0b",
            Severity::Low => "#3b82f6",
            Severity::Info => "#64748b",
        }
    }
}

/// A K8s hardening check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sHardeningCheck {
    pub id: String,
    pub category: K8sHardeningCategory,
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub remediation: String,
    pub severity: Severity,
    pub cis_benchmark: Option<String>,
    pub nsa_reference: Option<String>,
}

/// Result of a K8s hardening check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum K8sCheckResult {
    Pass,
    Fail,
    Warning,
    Error,
    Skipped,
}

/// K8s hardening finding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sHardeningFinding {
    pub id: String,
    pub audit_id: String,
    pub check_id: String,
    pub result: K8sCheckResult,
    pub severity: Severity,
    pub resource_type: Option<String>,
    pub resource_name: Option<String>,
    pub namespace: Option<String>,
    pub details: Option<String>,
    pub found_at: DateTime<Utc>,
}

/// K8s hardening audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sHardeningAudit {
    pub id: String,
    pub client_id: String,
    pub cluster_name: String,
    pub cluster_version: Option<String>,
    pub context_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: AuditStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Summary of K8s hardening audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sAuditSummary {
    pub audit_id: String,
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub category_results: Vec<K8sCategoryResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sCategoryResult {
    pub category: K8sHardeningCategory,
    pub display_name: String,
    pub color: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub score_percentage: f64,
}

// ============================================================================
// FinOps Calculator Models
// ============================================================================

/// Resource type for cost calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceType {
    Compute,
    Storage,
    Database,
    Network,
    Container,
    Serverless,
    Other,
}

impl ResourceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Compute => "Compute (VMs)",
            ResourceType::Storage => "Storage",
            ResourceType::Database => "Database",
            ResourceType::Network => "Network/Bandwidth",
            ResourceType::Container => "Containers/K8s",
            ResourceType::Serverless => "Serverless",
            ResourceType::Other => "Other",
        }
    }
}

/// On-premises resource for cost comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnPremResource {
    pub id: String,
    pub name: String,
    pub resource_type: ResourceType,
    pub description: Option<String>,
    // Compute specs
    pub vcpus: Option<u32>,
    pub memory_gb: Option<f64>,
    // Storage specs
    pub storage_gb: Option<f64>,
    pub storage_type: Option<String>, // SSD, HDD, NVMe
    pub iops: Option<u32>,
    // Network specs
    pub bandwidth_gbps: Option<f64>,
    pub egress_gb_month: Option<f64>,
    // Utilization
    pub avg_utilization_percent: Option<f64>,
    // Current costs
    pub monthly_cost: Option<f64>,
    pub annual_cost: Option<f64>,
}

/// Cloud cost estimate for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudCostEstimate {
    pub provider: CloudProvider,
    pub service_name: String,
    pub instance_type: Option<String>,
    pub region: String,
    pub monthly_cost: f64,
    pub annual_cost: f64,
    pub reserved_monthly: Option<f64>,  // 1-year reserved pricing
    pub reserved_annual: Option<f64>,
    pub spot_monthly: Option<f64>,      // Spot/preemptible pricing
    pub notes: Option<String>,
}

/// FinOps cost comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostComparison {
    pub id: String,
    pub client_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Resource with cloud cost estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCostAnalysis {
    pub resource: OnPremResource,
    pub estimates: Vec<CloudCostEstimate>,
    pub recommended_provider: Option<CloudProvider>,
    pub recommended_strategy: Option<MigrationStrategy>,
    pub potential_savings_percent: Option<f64>,
}

/// Complete FinOps summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinOpsSummary {
    pub comparison_id: String,
    pub total_on_prem_monthly: f64,
    pub total_on_prem_annual: f64,
    pub provider_estimates: Vec<ProviderEstimate>,
    pub recommended_provider: CloudProvider,
    pub potential_savings_monthly: f64,
    pub potential_savings_percent: f64,
    pub resource_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderEstimate {
    pub provider: CloudProvider,
    pub provider_name: String,
    pub monthly_cost: f64,
    pub annual_cost: f64,
    pub reserved_monthly: f64,
    pub reserved_annual: f64,
    pub savings_vs_onprem_percent: f64,
}
