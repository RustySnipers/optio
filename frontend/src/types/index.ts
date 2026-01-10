/**
 * Optio TypeScript Types
 *
 * Type-safe interfaces matching the Rust backend commands.
 * These ensure strict type safety between Rust and TypeScript.
 */

// ============================================================================
// Client Types
// ============================================================================

export interface Client {
  id: string;
  name: string;
  targetSubnet: string | null;
  contactEmail: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateClientRequest {
  name: string;
  targetSubnet?: string;
  contactEmail?: string;
  notes?: string;
}

export interface UpdateClientRequest {
  id: string;
  name: string;
  targetSubnet?: string;
  contactEmail?: string;
  notes?: string;
}

// ============================================================================
// Factory Types (Script Generation)
// ============================================================================

export interface ScriptConfigOptions {
  enableWinrm: boolean;
  configureDns: boolean;
  dnsServers?: string[];
  installAgent: boolean;
  agentInstaller?: string;
  enableFirewallLogging: boolean;
  customCommands?: string[];
}

export interface GenerateScriptRequest {
  clientId: string;
  clientName: string;
  targetSubnet: string;
  templateName: string;
  config: ScriptConfigOptions;
}

export interface GenerateScriptResponse {
  success: boolean;
  outputPath: string;
  scriptContent: string;
  scriptId: string;
  generatedAt: string;
  warnings: string[];
}

export interface TemplateInfo {
  name: string;
  description: string;
  category: string;
  requiredVars: string[];
  path: string;
}

export interface PreviewRequest {
  templateName: string;
  config: ScriptConfigOptions;
  clientName: string;
  targetSubnet: string;
}

export interface ValidateConfigRequest {
  clientName: string;
  targetSubnet: string;
  config: ScriptConfigOptions;
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

// ============================================================================
// System Types
// ============================================================================

export interface SystemInfo {
  osName: string;
  osVersion: string;
  hostname: string;
  username: string;
  appVersion: string;
  localIp: string | null;
}

// ============================================================================
// UI State Types
// ============================================================================

export type ViewMode = 'dashboard' | 'clients' | 'factory' | 'grc' | 'network' | 'settings';

export interface AppState {
  currentView: ViewMode;
  selectedClient: Client | null;
  isLoading: boolean;
  error: string | null;
}

export interface LogEntry {
  timestamp: Date;
  level: 'info' | 'warn' | 'error' | 'success';
  message: string;
}

// ============================================================================
// GRC Types (Governance, Risk, Compliance)
// ============================================================================

export type ComplianceStatus =
  | "NOT_ASSESSED"
  | "COMPLIANT"
  | "PARTIALLY_COMPLIANT"
  | "NON_COMPLIANT"
  | "NOT_APPLICABLE";

export type AssessmentStatus =
  | "DRAFT"
  | "IN_PROGRESS"
  | "UNDER_REVIEW"
  | "COMPLETED"
  | "ARCHIVED";

export type EvidenceType =
  | "DOCUMENT"
  | "SCREENSHOT"
  | "CONFIGURATION"
  | "SCAN_RESULT"
  | "INTERVIEW"
  | "LOG_FILE"
  | "OTHER";

export interface FrameworkInfo {
  id: string;
  name: string;
  description: string;
  controlCount: number;
  categories: CategoryInfo[];
}

export interface CategoryInfo {
  code: string;
  name: string;
  description: string;
  color: string;
}

export interface Control {
  id: string;
  framework: string;
  code: string;
  category: string;
  subcategory: string | null;
  title: string;
  description: string;
  guidance: string | null;
  crossReferences: string[];
  priority: number;
}

export interface Assessment {
  id: string;
  clientId: string;
  name: string;
  description: string | null;
  framework: string;
  scope: string | null;
  startedAt: string;
  completedAt: string | null;
  leadAssessor: string;
  status: AssessmentStatus;
}

export interface CreateAssessmentRequest {
  clientId: string;
  name: string;
  description?: string;
  framework: string;
  scope?: string;
  leadAssessor: string;
}

export interface ControlAssessment {
  id: string;
  assessmentId: string;
  controlId: string;
  status: ComplianceStatus;
  notes: string | null;
  gapDescription: string | null;
  remediation: string | null;
  remediationTarget: string | null;
  riskRating: number | null;
  evidenceIds: string[];
  assessedAt: string;
  assessedBy: string;
}

export interface UpdateControlAssessmentRequest {
  assessmentId: string;
  controlId: string;
  status: ComplianceStatus;
  notes?: string;
  gapDescription?: string;
  remediation?: string;
  remediationTarget?: string;
  riskRating?: number;
  assessedBy: string;
}

export interface Evidence {
  id: string;
  assessmentId: string;
  controlIds: string[];
  evidenceType: EvidenceType;
  title: string;
  description: string | null;
  filePath: string | null;
  url: string | null;
  fileHash: string | null;
  collectedAt: string;
  collectedBy: string;
  notes: string | null;
}

export interface CreateEvidenceRequest {
  assessmentId: string;
  controlIds: string[];
  evidenceType: EvidenceType;
  title: string;
  description?: string;
  filePath?: string;
  url?: string;
  notes?: string;
  collectedBy: string;
}

export interface CategoryScore {
  category: string;
  displayName: string;
  color: string;
  totalControls: number;
  compliant: number;
  partiallyCompliant: number;
  nonCompliant: number;
  notAssessed: number;
  notApplicable: number;
  compliancePercentage: number;
}

export interface AssessmentSummary {
  assessmentId: string;
  framework: string;
  overallCompliance: number;
  totalControls: number;
  compliant: number;
  partiallyCompliant: number;
  nonCompliant: number;
  notAssessed: number;
  notApplicable: number;
  categoryScores: CategoryScore[];
  highRiskGaps: number;
  evidenceCount: number;
}

// ============================================================================
// Error Types
// ============================================================================

export interface ErrorResponse {
  code: string;
  message: string;
  details?: string;
}
