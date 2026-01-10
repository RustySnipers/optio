//! Infrastructure Commands
//!
//! Tauri commands for Infrastructure & Migration module operations.
//! Includes Cloud Readiness Assessment, K8s Hardening Audit, and FinOps Calculator.

use crate::infrastructure::{
    models::*,
    cloud_readiness::{get_cloud_readiness_checklist, CloudReadinessAssessor},
    k8s_hardening::{get_k8s_hardening_checks, K8sHardeningAuditor},
    finops::{generate_finops_analysis, get_resource_templates, calculate_resource_cost, ResourceTemplate},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Cloud Readiness Commands
// ============================================================================

/// Get the full cloud readiness checklist
#[tauri::command]
pub async fn get_cloud_readiness_items() -> Result<Vec<CloudReadinessItem>, String> {
    Ok(get_cloud_readiness_checklist())
}

/// Get cloud readiness items filtered by category
#[tauri::command]
pub async fn get_cloud_readiness_by_category(
    category: String,
) -> Result<Vec<CloudReadinessItem>, String> {
    let cat = parse_readiness_category(&category)?;
    let items = get_cloud_readiness_checklist();
    Ok(items.into_iter().filter(|i| i.category == cat).collect())
}

/// Perform a cloud readiness assessment
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformReadinessAssessmentRequest {
    pub client_id: String,
    pub client_name: String,
    pub target_provider: String,
    pub item_statuses: Vec<ReadinessItemStatus>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessItemStatus {
    pub item_id: String,
    pub status: String,
    pub notes: Option<String>,
}

/// Perform cloud readiness assessment and get score
#[tauri::command]
pub async fn assess_cloud_readiness(
    request: PerformReadinessAssessmentRequest,
) -> Result<CloudReadinessAssessment, String> {
    let provider = parse_cloud_provider(&request.target_provider)?;
    let assessor = CloudReadinessAssessor::new(provider);

    // Build responses from the request
    let responses: Vec<ReadinessResponse> = request.item_statuses
        .into_iter()
        .map(|s| ReadinessResponse {
            item_id: s.item_id,
            status: parse_readiness_status(&s.status).unwrap_or(ReadinessStatus::NotStarted),
            notes: s.notes,
            blockers: None,
        })
        .collect();

    let assessment = assessor.assess(&request.client_id, &request.client_name, &responses);
    Ok(assessment)
}

// ============================================================================
// Kubernetes Hardening Commands
// ============================================================================

/// Get all K8s hardening checks
#[tauri::command]
pub async fn get_k8s_hardening_checklist() -> Result<Vec<K8sHardeningCheck>, String> {
    Ok(get_k8s_hardening_checks())
}

/// Get K8s hardening checks by category
#[tauri::command]
pub async fn get_k8s_hardening_by_category(
    category: String,
) -> Result<Vec<K8sHardeningCheck>, String> {
    let cat = parse_k8s_category(&category)?;
    let checks = get_k8s_hardening_checks();
    Ok(checks.into_iter().filter(|c| c.category == cat).collect())
}

/// Perform K8s hardening audit request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformK8sAuditRequest {
    pub client_id: String,
    pub cluster_name: String,
    pub cluster_version: String,
    pub check_results: Vec<K8sCheckResult>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sCheckResult {
    pub check_id: String,
    pub status: String,
    pub finding: Option<String>,
    pub affected_resources: Option<Vec<String>>,
}

/// Perform K8s hardening audit
#[tauri::command]
pub async fn audit_k8s_hardening(
    request: PerformK8sAuditRequest,
) -> Result<K8sHardeningAudit, String> {
    let auditor = K8sHardeningAuditor::new();

    // Build results from request
    let results: Vec<K8sCheckResultData> = request.check_results
        .into_iter()
        .map(|r| K8sCheckResultData {
            check_id: r.check_id,
            status: parse_k8s_check_status(&r.status).unwrap_or(K8sCheckStatus::NotChecked),
            finding: r.finding,
            affected_resources: r.affected_resources.unwrap_or_default(),
            remediation_applied: false,
        })
        .collect();

    let audit = auditor.perform_audit(
        &request.client_id,
        &request.cluster_name,
        &request.cluster_version,
        &results,
    );

    Ok(audit)
}

/// Get K8s hardening severity breakdown
#[tauri::command]
pub async fn get_k8s_severity_stats() -> Result<K8sSeverityStats, String> {
    let checks = get_k8s_hardening_checks();

    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for check in &checks {
        match check.severity {
            Severity::Critical => critical += 1,
            Severity::High => high += 1,
            Severity::Medium => medium += 1,
            Severity::Low => low += 1,
        }
    }

    Ok(K8sSeverityStats {
        total: checks.len(),
        critical,
        high,
        medium,
        low,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sSeverityStats {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

// ============================================================================
// FinOps Commands
// ============================================================================

/// Get resource templates for quick estimation
#[tauri::command]
pub async fn get_finops_templates() -> Result<Vec<FinOpsTemplate>, String> {
    let templates = get_resource_templates();
    Ok(templates
        .into_iter()
        .map(|t| FinOpsTemplate {
            name: t.name,
            description: t.description,
            resource_count: t.resources.len(),
        })
        .collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinOpsTemplate {
    pub name: String,
    pub description: String,
    pub resource_count: usize,
}

/// Calculate cost for a single resource
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateResourceCostRequest {
    pub resource_type: String,
    pub name: String,
    pub quantity: u32,
    pub vcpus: Option<u32>,
    pub memory_gb: Option<f64>,
    pub storage_gb: Option<f64>,
    pub bandwidth_gbps: Option<f64>,
    pub iops: Option<u32>,
    pub provider: String,
}

#[tauri::command]
pub async fn calculate_single_resource_cost(
    request: CalculateResourceCostRequest,
) -> Result<f64, String> {
    let provider = parse_cloud_provider(&request.provider)?;
    let resource_type = parse_resource_type(&request.resource_type)?;

    let resource = ResourceCostEstimate {
        resource_type,
        name: request.name,
        quantity: request.quantity,
        specs: ResourceSpecs {
            vcpus: request.vcpus,
            memory_gb: request.memory_gb,
            storage_gb: request.storage_gb,
            bandwidth_gbps: request.bandwidth_gbps,
            iops: request.iops,
        },
        monthly_cost: 0.0,
        notes: None,
    };

    let cost = calculate_resource_cost(&resource, &provider);
    Ok((cost * 100.0).round() / 100.0) // Round to 2 decimal places
}

/// Generate full FinOps analysis
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateFinOpsAnalysisRequest {
    pub client_id: String,
    pub target_provider: String,
    pub migration_strategy: String,
    pub current_costs: OnPremCostsInput,
    pub resources: Vec<ResourceInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnPremCostsInput {
    pub hardware_monthly: f64,
    pub software_licensing_monthly: f64,
    pub datacenter_monthly: f64,
    pub personnel_monthly: f64,
    pub maintenance_monthly: f64,
    pub power_cooling_monthly: f64,
    pub network_monthly: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceInput {
    pub resource_type: String,
    pub name: String,
    pub quantity: u32,
    pub vcpus: Option<u32>,
    pub memory_gb: Option<f64>,
    pub storage_gb: Option<f64>,
    pub bandwidth_gbps: Option<f64>,
    pub iops: Option<u32>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn generate_finops_report(
    request: GenerateFinOpsAnalysisRequest,
) -> Result<FinOpsAnalysis, String> {
    let provider = parse_cloud_provider(&request.target_provider)?;
    let strategy = parse_migration_strategy(&request.migration_strategy)?;

    let current_costs = OnPremiseCosts {
        hardware_monthly: request.current_costs.hardware_monthly,
        software_licensing_monthly: request.current_costs.software_licensing_monthly,
        datacenter_monthly: request.current_costs.datacenter_monthly,
        personnel_monthly: request.current_costs.personnel_monthly,
        maintenance_monthly: request.current_costs.maintenance_monthly,
        power_cooling_monthly: request.current_costs.power_cooling_monthly,
        network_monthly: request.current_costs.network_monthly,
    };

    let resources: Result<Vec<ResourceCostEstimate>, String> = request.resources
        .into_iter()
        .map(|r| {
            let resource_type = parse_resource_type(&r.resource_type)?;
            Ok(ResourceCostEstimate {
                resource_type,
                name: r.name,
                quantity: r.quantity,
                specs: ResourceSpecs {
                    vcpus: r.vcpus,
                    memory_gb: r.memory_gb,
                    storage_gb: r.storage_gb,
                    bandwidth_gbps: r.bandwidth_gbps,
                    iops: r.iops,
                },
                monthly_cost: 0.0,
                notes: r.notes,
            })
        })
        .collect();

    let resources = resources?;

    let mut analysis = generate_finops_analysis(&current_costs, &resources, &provider, &strategy);
    analysis.client_id = request.client_id;

    Ok(analysis)
}

/// Compare costs across providers
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareProvidersRequest {
    pub resources: Vec<ResourceInput>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderComparison {
    pub provider: String,
    pub monthly_cost: f64,
    pub annual_cost: f64,
}

#[tauri::command]
pub async fn compare_cloud_providers(
    request: CompareProvidersRequest,
) -> Result<Vec<ProviderComparison>, String> {
    let providers = vec![CloudProvider::AWS, CloudProvider::Azure, CloudProvider::GCP];

    let resources: Result<Vec<ResourceCostEstimate>, String> = request.resources
        .iter()
        .map(|r| {
            let resource_type = parse_resource_type(&r.resource_type)?;
            Ok(ResourceCostEstimate {
                resource_type,
                name: r.name.clone(),
                quantity: r.quantity,
                specs: ResourceSpecs {
                    vcpus: r.vcpus,
                    memory_gb: r.memory_gb,
                    storage_gb: r.storage_gb,
                    bandwidth_gbps: r.bandwidth_gbps,
                    iops: r.iops,
                },
                monthly_cost: 0.0,
                notes: r.notes.clone(),
            })
        })
        .collect();

    let resources = resources?;

    let comparisons = providers
        .into_iter()
        .map(|provider| {
            let monthly: f64 = resources
                .iter()
                .map(|r| calculate_resource_cost(r, &provider))
                .sum();

            ProviderComparison {
                provider: format!("{:?}", provider),
                monthly_cost: (monthly * 100.0).round() / 100.0,
                annual_cost: (monthly * 12.0 * 100.0).round() / 100.0,
            }
        })
        .collect();

    Ok(comparisons)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_cloud_provider(s: &str) -> Result<CloudProvider, String> {
    match s.to_uppercase().as_str() {
        "AWS" | "AMAZON" => Ok(CloudProvider::AWS),
        "AZURE" | "MICROSOFT" => Ok(CloudProvider::Azure),
        "GCP" | "GOOGLE" => Ok(CloudProvider::GCP),
        _ => Err(format!("Unknown cloud provider: {}", s)),
    }
}

fn parse_migration_strategy(s: &str) -> Result<MigrationStrategy, String> {
    match s.to_uppercase().as_str() {
        "REHOST" | "LIFT_AND_SHIFT" | "LIFTANDSHIFT" => Ok(MigrationStrategy::Rehost),
        "REPLATFORM" => Ok(MigrationStrategy::Replatform),
        "REFACTOR" | "REARCHITECT" => Ok(MigrationStrategy::Refactor),
        "REPURCHASE" | "REPLACE" => Ok(MigrationStrategy::Repurchase),
        "RETIRE" => Ok(MigrationStrategy::Retire),
        "RETAIN" => Ok(MigrationStrategy::Retain),
        _ => Err(format!("Unknown migration strategy: {}", s)),
    }
}

fn parse_resource_type(s: &str) -> Result<ResourceType, String> {
    match s.to_uppercase().as_str() {
        "VIRTUALMACHINE" | "VIRTUAL_MACHINE" | "VM" => Ok(ResourceType::VirtualMachine),
        "CONTAINER" => Ok(ResourceType::Container),
        "DATABASE" | "DB" => Ok(ResourceType::Database),
        "STORAGE" => Ok(ResourceType::Storage),
        "NETWORK" => Ok(ResourceType::Network),
        "KUBERNETES" | "K8S" => Ok(ResourceType::Kubernetes),
        "SERVERLESS" | "LAMBDA" | "FUNCTION" => Ok(ResourceType::Serverless),
        "LOADBALANCER" | "LOAD_BALANCER" | "LB" => Ok(ResourceType::LoadBalancer),
        "OTHER" => Ok(ResourceType::Other),
        _ => Err(format!("Unknown resource type: {}", s)),
    }
}

fn parse_readiness_category(s: &str) -> Result<ReadinessCategory, String> {
    match s.to_uppercase().as_str() {
        "BUSINESSALIGNMENT" | "BUSINESS_ALIGNMENT" | "BUSINESS" => Ok(ReadinessCategory::BusinessAlignment),
        "TECHNICALREADINESS" | "TECHNICAL_READINESS" | "TECHNICAL" => Ok(ReadinessCategory::TechnicalReadiness),
        "SECURITYCOMPLIANCE" | "SECURITY_COMPLIANCE" | "SECURITY" => Ok(ReadinessCategory::SecurityCompliance),
        "OPERATIONALREADINESS" | "OPERATIONAL_READINESS" | "OPERATIONAL" => Ok(ReadinessCategory::OperationalReadiness),
        "FINANCIALPLANNING" | "FINANCIAL_PLANNING" | "FINANCIAL" => Ok(ReadinessCategory::FinancialPlanning),
        "PEOPLEPROCESS" | "PEOPLE_PROCESS" | "PEOPLE" => Ok(ReadinessCategory::PeopleProcess),
        "DATAMANAGEMENT" | "DATA_MANAGEMENT" | "DATA" => Ok(ReadinessCategory::DataManagement),
        _ => Err(format!("Unknown readiness category: {}", s)),
    }
}

fn parse_readiness_status(s: &str) -> Result<ReadinessStatus, String> {
    match s.to_uppercase().as_str() {
        "NOTSTARTED" | "NOT_STARTED" => Ok(ReadinessStatus::NotStarted),
        "INPROGRESS" | "IN_PROGRESS" => Ok(ReadinessStatus::InProgress),
        "COMPLETED" | "COMPLETE" => Ok(ReadinessStatus::Completed),
        "BLOCKED" => Ok(ReadinessStatus::Blocked),
        "NOTAPPLICABLE" | "NOT_APPLICABLE" | "NA" | "N/A" => Ok(ReadinessStatus::NotApplicable),
        _ => Err(format!("Unknown readiness status: {}", s)),
    }
}

fn parse_k8s_category(s: &str) -> Result<K8sHardeningCategory, String> {
    match s.to_uppercase().as_str() {
        "PODSECURITY" | "POD_SECURITY" | "POD" => Ok(K8sHardeningCategory::PodSecurity),
        "NETWORKPOLICIES" | "NETWORK_POLICIES" | "NETWORK" => Ok(K8sHardeningCategory::NetworkPolicies),
        "AUTHENTICATION" | "AUTH" => Ok(K8sHardeningCategory::Authentication),
        "AUTHORIZATION" | "AUTHZ" | "RBAC" => Ok(K8sHardeningCategory::Authorization),
        "LOGGING" | "AUDIT" => Ok(K8sHardeningCategory::Logging),
        "THREATDETECTION" | "THREAT_DETECTION" | "THREAT" => Ok(K8sHardeningCategory::ThreatDetection),
        "SUPPLYCHAIN" | "SUPPLY_CHAIN" | "SUPPLY" => Ok(K8sHardeningCategory::SupplyChain),
        "SECRETS" | "SECRET" => Ok(K8sHardeningCategory::Secrets),
        _ => Err(format!("Unknown K8s hardening category: {}", s)),
    }
}

fn parse_k8s_check_status(s: &str) -> Result<K8sCheckStatus, String> {
    match s.to_uppercase().as_str() {
        "PASS" | "PASSED" => Ok(K8sCheckStatus::Pass),
        "FAIL" | "FAILED" => Ok(K8sCheckStatus::Fail),
        "WARN" | "WARNING" => Ok(K8sCheckStatus::Warn),
        "NOTCHECKED" | "NOT_CHECKED" | "SKIP" | "SKIPPED" => Ok(K8sCheckStatus::NotChecked),
        _ => Err(format!("Unknown K8s check status: {}", s)),
    }
}
