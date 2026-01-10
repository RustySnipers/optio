/**
 * Tauri Command Bindings
 *
 * Type-safe wrappers for Rust backend commands via Tauri IPC.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  Client,
  CreateClientRequest,
  UpdateClientRequest,
  GenerateScriptRequest,
  GenerateScriptResponse,
  TemplateInfo,
  PreviewRequest,
  ValidateConfigRequest,
  ValidationResult,
  SystemInfo,
  FrameworkInfo,
  Control,
  Assessment,
  CreateAssessmentRequest,
  ControlAssessment,
  UpdateControlAssessmentRequest,
  Evidence,
  CreateEvidenceRequest,
  AssessmentSummary,
  CloudReadinessItem,
  CloudReadinessAssessment,
  PerformReadinessAssessmentRequest,
  K8sHardeningCheck,
  K8sHardeningAudit,
  PerformK8sAuditRequest,
  K8sSeverityStats,
  FinOpsTemplate,
  FinOpsAnalysis,
  GenerateFinOpsAnalysisRequest,
  CalculateResourceCostRequest,
  ProviderComparison,
  CompareProvidersRequest,
  NmapInfo,
  ScanTypeInfo,
  CommonPort,
  TargetValidation,
  ScanJob,
  CreateScanRequest,
  Asset,
  AssetGroup,
  UpdateAssetRequest,
  CreateGroupRequest,
  NetworkStats,
} from "@/types";

// ============================================================================
// Factory Commands
// ============================================================================

/**
 * Generate a client provisioning script
 */
export async function generateClientScript(
  request: GenerateScriptRequest
): Promise<GenerateScriptResponse> {
  return invoke<GenerateScriptResponse>("generate_client_script", { request });
}

/**
 * List available script templates
 */
export async function listTemplates(): Promise<TemplateInfo[]> {
  return invoke<TemplateInfo[]>("list_templates");
}

/**
 * Get a preview of the generated script
 */
export async function getScriptPreview(request: PreviewRequest): Promise<string> {
  return invoke<string>("get_script_preview", { request });
}

/**
 * Validate script configuration
 */
export async function validateConfig(
  request: ValidateConfigRequest
): Promise<ValidationResult> {
  return invoke<ValidationResult>("validate_config", { request });
}

// ============================================================================
// Client Commands
// ============================================================================

/**
 * Create a new client
 */
export async function createClient(request: CreateClientRequest): Promise<Client> {
  return invoke<Client>("create_client", { request });
}

/**
 * List all clients
 */
export async function listClients(): Promise<Client[]> {
  return invoke<Client[]>("list_clients");
}

/**
 * Get a single client by ID
 */
export async function getClient(id: string): Promise<Client> {
  return invoke<Client>("get_client", { id });
}

/**
 * Update an existing client
 */
export async function updateClient(request: UpdateClientRequest): Promise<Client> {
  return invoke<Client>("update_client", { request });
}

/**
 * Delete a client
 */
export async function deleteClient(id: string): Promise<boolean> {
  return invoke<boolean>("delete_client", { id });
}

// ============================================================================
// System Commands
// ============================================================================

/**
 * Get system information
 */
export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>("get_system_info");
}

/**
 * Get consultant's local IP address
 */
export async function getConsultantIp(): Promise<string> {
  return invoke<string>("get_consultant_ip");
}

// ============================================================================
// GRC Commands (Governance, Risk, Compliance)
// ============================================================================

/**
 * List available compliance frameworks
 */
export async function listFrameworks(): Promise<FrameworkInfo[]> {
  return invoke<FrameworkInfo[]>("list_frameworks");
}

/**
 * Get all controls for a specific framework
 */
export async function getFrameworkControls(framework: string): Promise<Control[]> {
  return invoke<Control[]>("get_framework_controls_cmd", { framework });
}

/**
 * Create a new assessment
 */
export async function createAssessment(
  request: CreateAssessmentRequest
): Promise<Assessment> {
  return invoke<Assessment>("create_assessment", { request });
}

/**
 * Get an assessment by ID
 */
export async function getAssessment(id: string): Promise<Assessment | null> {
  return invoke<Assessment | null>("get_assessment", { id });
}

/**
 * List assessments for a client
 */
export async function listClientAssessments(clientId: string): Promise<Assessment[]> {
  return invoke<Assessment[]>("list_client_assessments", { clientId });
}

/**
 * List all assessments
 */
export async function listAssessments(): Promise<Assessment[]> {
  return invoke<Assessment[]>("list_assessments");
}

/**
 * Update assessment status
 */
export async function updateAssessmentStatus(
  id: string,
  status: string
): Promise<boolean> {
  return invoke<boolean>("update_assessment_status", { id, status });
}

/**
 * Delete an assessment
 */
export async function deleteAssessment(id: string): Promise<boolean> {
  return invoke<boolean>("delete_assessment", { id });
}

/**
 * Update a control's assessment status
 */
export async function updateControlAssessment(
  request: UpdateControlAssessmentRequest
): Promise<ControlAssessment> {
  return invoke<ControlAssessment>("update_control_assessment", { request });
}

/**
 * Get all control assessments for an assessment
 */
export async function getControlAssessments(
  assessmentId: string
): Promise<ControlAssessment[]> {
  return invoke<ControlAssessment[]>("get_control_assessments", { assessmentId });
}

/**
 * Batch update multiple controls
 */
export async function batchUpdateControls(
  assessmentId: string,
  controlIds: string[],
  status: string,
  assessedBy: string
): Promise<number> {
  return invoke<number>("batch_update_controls", {
    request: { assessmentId, controlIds, status, assessedBy },
  });
}

/**
 * Create evidence for an assessment
 */
export async function createEvidence(
  request: CreateEvidenceRequest
): Promise<Evidence> {
  return invoke<Evidence>("create_evidence", { request });
}

/**
 * Get all evidence for an assessment
 */
export async function getAssessmentEvidence(
  assessmentId: string
): Promise<Evidence[]> {
  return invoke<Evidence[]>("get_assessment_evidence", { assessmentId });
}

/**
 * Delete evidence
 */
export async function deleteEvidence(id: string): Promise<boolean> {
  return invoke<boolean>("delete_evidence", { id });
}

/**
 * Get assessment summary with compliance scores
 */
export async function getAssessmentSummary(
  assessmentId: string
): Promise<AssessmentSummary> {
  return invoke<AssessmentSummary>("get_assessment_summary", { assessmentId });
}

// ============================================================================
// Infrastructure Commands (Cloud Migration & K8s Hardening)
// ============================================================================

/**
 * Get all cloud readiness checklist items
 */
export async function getCloudReadinessItems(): Promise<CloudReadinessItem[]> {
  return invoke<CloudReadinessItem[]>("get_cloud_readiness_items");
}

/**
 * Get cloud readiness items by category
 */
export async function getCloudReadinessByCategory(
  category: string
): Promise<CloudReadinessItem[]> {
  return invoke<CloudReadinessItem[]>("get_cloud_readiness_by_category", { category });
}

/**
 * Perform cloud readiness assessment
 */
export async function assessCloudReadiness(
  request: PerformReadinessAssessmentRequest
): Promise<CloudReadinessAssessment> {
  return invoke<CloudReadinessAssessment>("assess_cloud_readiness", { request });
}

/**
 * Get all K8s hardening checks
 */
export async function getK8sHardeningChecklist(): Promise<K8sHardeningCheck[]> {
  return invoke<K8sHardeningCheck[]>("get_k8s_hardening_checklist");
}

/**
 * Get K8s hardening checks by category
 */
export async function getK8sHardeningByCategory(
  category: string
): Promise<K8sHardeningCheck[]> {
  return invoke<K8sHardeningCheck[]>("get_k8s_hardening_by_category", { category });
}

/**
 * Perform K8s hardening audit
 */
export async function auditK8sHardening(
  request: PerformK8sAuditRequest
): Promise<K8sHardeningAudit> {
  return invoke<K8sHardeningAudit>("audit_k8s_hardening", { request });
}

/**
 * Get K8s severity statistics
 */
export async function getK8sSeverityStats(): Promise<K8sSeverityStats> {
  return invoke<K8sSeverityStats>("get_k8s_severity_stats");
}

/**
 * Get FinOps resource templates
 */
export async function getFinOpsTemplates(): Promise<FinOpsTemplate[]> {
  return invoke<FinOpsTemplate[]>("get_finops_templates");
}

/**
 * Calculate cost for a single resource
 */
export async function calculateSingleResourceCost(
  request: CalculateResourceCostRequest
): Promise<number> {
  return invoke<number>("calculate_single_resource_cost", { request });
}

/**
 * Generate full FinOps analysis
 */
export async function generateFinOpsReport(
  request: GenerateFinOpsAnalysisRequest
): Promise<FinOpsAnalysis> {
  return invoke<FinOpsAnalysis>("generate_finops_report", { request });
}

/**
 * Compare costs across cloud providers
 */
export async function compareCloudProviders(
  request: CompareProvidersRequest
): Promise<ProviderComparison[]> {
  return invoke<ProviderComparison[]>("compare_cloud_providers", { request });
}

// ============================================================================
// Network Intelligence Commands
// ============================================================================

/**
 * Check if Nmap is installed
 */
export async function checkNmap(): Promise<NmapInfo> {
  return invoke<NmapInfo>("check_nmap");
}

/**
 * Get available scan types
 */
export async function getScanTypeList(): Promise<ScanTypeInfo[]> {
  return invoke<ScanTypeInfo[]>("get_scan_type_list");
}

/**
 * Get common ports reference
 */
export async function getCommonPortList(): Promise<CommonPort[]> {
  return invoke<CommonPort[]>("get_common_port_list");
}

/**
 * Validate a scan target
 */
export async function validateScanTarget(target: string): Promise<TargetValidation> {
  return invoke<TargetValidation>("validate_scan_target", { target });
}

/**
 * Create a new scan job
 */
export async function createScan(request: CreateScanRequest): Promise<ScanJob> {
  return invoke<ScanJob>("create_scan", { request });
}

/**
 * Preview the Nmap command that would be executed
 */
export async function previewScanCommand(
  targets: string[],
  scanType: string,
  ports?: string,
  aggressive?: boolean
): Promise<string> {
  return invoke<string>("preview_scan_command", {
    targets,
    scanType,
    ports,
    aggressive: aggressive ?? false,
  });
}

/**
 * List all scans for a client
 */
export async function listScans(clientId: string): Promise<ScanJob[]> {
  return invoke<ScanJob[]>("list_scans", { clientId });
}

/**
 * Get a specific scan by ID
 */
export async function getScan(scanId: string): Promise<ScanJob | null> {
  return invoke<ScanJob | null>("get_scan", { scanId });
}

/**
 * Delete a scan
 */
export async function deleteScan(scanId: string): Promise<boolean> {
  return invoke<boolean>("delete_scan", { scanId });
}

/**
 * Get all assets for a client
 */
export async function listAssets(clientId: string): Promise<Asset[]> {
  return invoke<Asset[]>("list_assets", { clientId });
}

/**
 * Get demo assets for development
 */
export async function getDemoAssets(clientId: string): Promise<Asset[]> {
  return invoke<Asset[]>("get_demo_assets", { clientId });
}

/**
 * Get a specific asset by ID
 */
export async function getAsset(assetId: string): Promise<Asset | null> {
  return invoke<Asset | null>("get_asset", { assetId });
}

/**
 * Update an asset
 */
export async function updateAsset(request: UpdateAssetRequest): Promise<Asset> {
  return invoke<Asset>("update_asset", { request });
}

/**
 * Delete an asset
 */
export async function deleteAsset(assetId: string): Promise<boolean> {
  return invoke<boolean>("delete_asset", { assetId });
}

/**
 * Get network statistics for a client
 */
export async function getNetworkStats(clientId: string): Promise<NetworkStats> {
  return invoke<NetworkStats>("get_network_stats", { clientId });
}

/**
 * Create a new asset group
 */
export async function createAssetGroup(request: CreateGroupRequest): Promise<AssetGroup> {
  return invoke<AssetGroup>("create_asset_group", { request });
}

/**
 * List all asset groups for a client
 */
export async function listAssetGroups(clientId: string): Promise<AssetGroup[]> {
  return invoke<AssetGroup[]>("list_asset_groups", { clientId });
}

/**
 * Add asset to a group
 */
export async function addAssetToGroup(groupId: string, assetId: string): Promise<void> {
  return invoke<void>("add_asset_to_group", { groupId, assetId });
}

/**
 * Remove asset from a group
 */
export async function removeAssetFromGroup(groupId: string, assetId: string): Promise<void> {
  return invoke<void>("remove_asset_from_group", { groupId, assetId });
}
