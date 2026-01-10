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
