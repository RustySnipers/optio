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

export type ViewMode = 'dashboard' | 'clients' | 'factory' | 'grc' | 'infrastructure' | 'network' | 'reporting' | 'settings';

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
// Infrastructure Types (Cloud Migration & K8s Hardening)
// ============================================================================

export type CloudProvider = "AWS" | "Azure" | "GCP";

export type MigrationStrategy =
  | "Rehost"
  | "Replatform"
  | "Refactor"
  | "Repurchase"
  | "Retire"
  | "Retain";

export type ReadinessCategory =
  | "BusinessAlignment"
  | "TechnicalReadiness"
  | "SecurityCompliance"
  | "OperationalReadiness"
  | "FinancialPlanning"
  | "PeopleProcess"
  | "DataManagement";

export type ReadinessStatus =
  | "NotStarted"
  | "InProgress"
  | "Completed"
  | "Blocked"
  | "NotApplicable";

export type K8sHardeningCategory =
  | "PodSecurity"
  | "NetworkPolicies"
  | "Authentication"
  | "Authorization"
  | "Logging"
  | "ThreatDetection"
  | "SupplyChain"
  | "Secrets";

export type Severity = "Critical" | "High" | "Medium" | "Low";

export type K8sCheckStatus = "Pass" | "Fail" | "Warn" | "NotChecked";

export type ResourceType =
  | "VirtualMachine"
  | "Container"
  | "Database"
  | "Storage"
  | "Network"
  | "Kubernetes"
  | "Serverless"
  | "LoadBalancer"
  | "Other";

// Cloud Readiness Types
export interface CloudReadinessItem {
  id: string;
  category: ReadinessCategory;
  title: string;
  description: string;
  guidance: string;
  priority: number;
  dependencies: string[];
  estimatedEffortDays: number;
}

export interface ReadinessResponse {
  itemId: string;
  status: ReadinessStatus;
  notes: string | null;
  blockers: string | null;
}

export interface CloudReadinessAssessment {
  id: string;
  clientId: string;
  clientName: string;
  targetProvider: CloudProvider;
  assessmentDate: string;
  responses: ReadinessResponse[];
  overallScore: number;
  categoryScores: Record<string, number>;
  criticalBlockers: string[];
  recommendations: string[];
}

export interface PerformReadinessAssessmentRequest {
  clientId: string;
  clientName: string;
  targetProvider: string;
  itemStatuses: ReadinessItemStatus[];
}

export interface ReadinessItemStatus {
  itemId: string;
  status: string;
  notes?: string;
}

// K8s Hardening Types
export interface K8sHardeningCheck {
  id: string;
  category: K8sHardeningCategory;
  title: string;
  description: string;
  rationale: string;
  remediation: string;
  severity: Severity;
  references: string[];
  automatable: boolean;
}

export interface K8sCheckResultData {
  checkId: string;
  status: K8sCheckStatus;
  finding: string | null;
  affectedResources: string[];
  remediationApplied: boolean;
}

export interface K8sHardeningAudit {
  id: string;
  clientId: string;
  clusterName: string;
  clusterVersion: string;
  auditDate: string;
  results: K8sCheckResultData[];
  overallScore: number;
  criticalFindings: number;
  highFindings: number;
  mediumFindings: number;
  lowFindings: number;
  recommendations: string[];
}

export interface PerformK8sAuditRequest {
  clientId: string;
  clusterName: string;
  clusterVersion: string;
  checkResults: K8sCheckResult[];
}

export interface K8sCheckResult {
  checkId: string;
  status: string;
  finding?: string;
  affectedResources?: string[];
}

export interface K8sSeverityStats {
  total: number;
  critical: number;
  high: number;
  medium: number;
  low: number;
}

// FinOps Types
export interface ResourceSpecs {
  vcpus?: number;
  memoryGb?: number;
  storageGb?: number;
  bandwidthGbps?: number;
  iops?: number;
}

export interface ResourceCostEstimate {
  resourceType: ResourceType;
  name: string;
  quantity: number;
  specs: ResourceSpecs;
  monthlyCost: number;
  notes: string | null;
}

export interface OnPremiseCosts {
  hardwareMonthly: number;
  softwareLicensingMonthly: number;
  datacenterMonthly: number;
  personnelMonthly: number;
  maintenanceMonthly: number;
  powerCoolingMonthly: number;
  networkMonthly: number;
}

export interface CostRecommendation {
  category: string;
  title: string;
  description: string;
  estimatedSavings: number;
  effort: string;
  priority: number;
}

export interface FinOpsAnalysis {
  id: string;
  clientId: string;
  analysisDate: string;
  targetProvider: CloudProvider;
  migrationStrategy: MigrationStrategy;
  currentMonthlyCost: number;
  projectedMonthlyCost: number;
  estimatedSavingsPercentage: number;
  migrationCostEstimate: number;
  roiMonths: number;
  resourceBreakdown: ResourceCostEstimate[];
  recommendations: CostRecommendation[];
  assumptions: string[];
}

export interface FinOpsTemplate {
  name: string;
  description: string;
  resourceCount: number;
}

export interface GenerateFinOpsAnalysisRequest {
  clientId: string;
  targetProvider: string;
  migrationStrategy: string;
  currentCosts: OnPremCostsInput;
  resources: ResourceInput[];
}

export interface OnPremCostsInput {
  hardwareMonthly: number;
  softwareLicensingMonthly: number;
  datacenterMonthly: number;
  personnelMonthly: number;
  maintenanceMonthly: number;
  powerCoolingMonthly: number;
  networkMonthly: number;
}

export interface ResourceInput {
  resourceType: string;
  name: string;
  quantity: number;
  vcpus?: number;
  memoryGb?: number;
  storageGb?: number;
  bandwidthGbps?: number;
  iops?: number;
  notes?: string;
}

export interface CalculateResourceCostRequest {
  resourceType: string;
  name: string;
  quantity: number;
  vcpus?: number;
  memoryGb?: number;
  storageGb?: number;
  bandwidthGbps?: number;
  iops?: number;
  provider: string;
}

export interface ProviderComparison {
  provider: string;
  monthlyCost: number;
  annualCost: number;
}

export interface CompareProvidersRequest {
  resources: ResourceInput[];
}

// ============================================================================
// Network Intelligence Types
// ============================================================================

export type ScanType =
  | "ping_sweep"
  | "quick_scan"
  | "standard_scan"
  | "full_scan"
  | "service_detection"
  | "os_detection"
  | "vulnerability_scan"
  | "udp_scan"
  | "custom";

export type ScanStatus =
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export type PortState =
  | "open"
  | "closed"
  | "filtered"
  | "unfiltered"
  | "open_filtered"
  | "closed_filtered";

export type Protocol = "tcp" | "udp" | "sctp";

export type AssetCategory =
  | "server"
  | "workstation"
  | "network_device"
  | "security_device"
  | "printer"
  | "iot"
  | "mobile"
  | "virtual"
  | "cloud"
  | "unknown";

export type AssetCriticality =
  | "critical"
  | "high"
  | "medium"
  | "low"
  | "informational";

export type AssetStatus =
  | "active"
  | "inactive"
  | "decommissioned"
  | "pending"
  | "maintenance";

export interface NmapInfo {
  installed: boolean;
  version: string | null;
  path: string | null;
}

export interface ScanTypeInfo {
  scanType: ScanType;
  name: string;
  description: string;
  duration: string;
  requiresRoot: boolean;
}

export interface CommonPort {
  port: number;
  service: string;
  description: string;
}

export interface TargetValidation {
  valid: boolean;
  targetType: string | null;
  normalized: string | null;
  error: string | null;
}

export interface ScanConfig {
  targets: string[];
  scanType: ScanType;
  customArgs?: string;
  ports?: string;
  excludeTargets?: string[];
  aggressive: boolean;
  skipDiscovery: boolean;
}

export interface ScanJob {
  id: string;
  clientId: string;
  name: string;
  config: ScanConfig;
  status: ScanStatus;
  createdAt: string;
  startedAt: string | null;
  completedAt: string | null;
  error: string | null;
  progress: number;
  rawOutput: string | null;
}

export interface CreateScanRequest {
  clientId: string;
  name: string;
  targets: string[];
  scanType: string;
  customArgs?: string;
  ports?: string;
  excludeTargets?: string[];
  aggressive: boolean;
  skipDiscovery: boolean;
}

export interface DiscoveredPort {
  port: number;
  protocol: Protocol;
  state: PortState;
  service: string | null;
  product: string | null;
  version: string | null;
  extraInfo: string | null;
}

export interface OsMatch {
  name: string;
  accuracy: number;
  osFamily: string | null;
  osGen: string | null;
  deviceType: string | null;
}

export interface DiscoveredHost {
  ipAddress: string;
  macAddress: string | null;
  hostname: string | null;
  vendor: string | null;
  status: string;
  ports: DiscoveredPort[];
  osMatches: OsMatch[];
}

export interface AssetService {
  port: number;
  protocol: Protocol;
  name: string;
  version: string | null;
  state: PortState;
}

export interface Asset {
  id: string;
  clientId: string;
  name: string;
  ipAddress: string;
  macAddress: string | null;
  category: AssetCategory;
  operatingSystem: string | null;
  criticality: AssetCriticality;
  status: AssetStatus;
  location: string | null;
  owner: string | null;
  description: string | null;
  services: AssetService[];
  tags: string[];
  firstSeen: string;
  lastSeen: string;
  scanIds: string[];
}

export interface AssetGroup {
  id: string;
  clientId: string;
  name: string;
  description: string | null;
  assetIds: string[];
  color: string | null;
}

export interface UpdateAssetRequest {
  id: string;
  name: string;
  category: string;
  criticality: string;
  status: string;
  location?: string;
  owner?: string;
  description?: string;
  tags: string[];
}

export interface CreateGroupRequest {
  clientId: string;
  name: string;
  description?: string;
}

export interface CategoryCount {
  category: AssetCategory;
  count: number;
}

export interface CriticalityCount {
  criticality: AssetCriticality;
  count: number;
}

export interface ServiceCount {
  service: string;
  port: number;
  count: number;
}

export interface ScanSummary {
  id: string;
  name: string;
  scanType: ScanType;
  status: ScanStatus;
  hostsFound: number;
  completedAt: string | null;
}

export interface NetworkStats {
  totalAssets: number;
  activeAssets: number;
  totalScans: number;
  byCategory: CategoryCount[];
  byCriticality: CriticalityCount[];
  topServices: ServiceCount[];
  recentScans: ScanSummary[];
}

// ============================================================================
// Reporting Types
// ============================================================================

export type ReportType =
  | "ExecutiveSummary"
  | "TechnicalAssessment"
  | "ComplianceReport"
  | "NetworkAssessment"
  | "CloudReadiness"
  | "SecurityFindings"
  | "FullEngagement";

export type ExportFormat = "Pdf" | "Html" | "Markdown" | "Docx" | "Json";

export type ReportStatus =
  | "Draft"
  | "Generating"
  | "Ready"
  | "Error"
  | "Archived";

export interface ReportTemplate {
  reportType: ReportType;
  name: string;
  description: string;
  sections: ReportSectionDef[];
}

export interface ReportSectionDef {
  id: string;
  title: string;
  description: string;
  required: boolean;
  order: number;
}

export interface ReportTypeInfo {
  reportType: ReportType;
  name: string;
  description: string;
  icon: string;
}

export interface ExportFormatInfo {
  format: ExportFormat;
  name: string;
  extension: string;
  mimeType: string;
}

export interface GenerateReportRequest {
  reportType: string;
  clientId: string;
  clientName: string;
  title: string;
  subtitle?: string;
  author: string;
  organization?: string;
  format: string;
  includeToc: boolean;
  includeExecutiveSummary: boolean;
  includeAppendices: boolean;
  includeCharts: boolean;
  classification?: string;
  notes?: string;
}

export interface ReportConfig {
  reportType: ReportType;
  clientId: string;
  clientName: string;
  title: string;
  subtitle: string | null;
  author: string;
  organization: string | null;
  format: ExportFormat;
  includeToc: boolean;
  includeExecutiveSummary: boolean;
  includeAppendices: boolean;
  includeCharts: boolean;
  logoPath: string | null;
  primaryColor: string | null;
  notes: string | null;
  classification: string | null;
  dataSources: string[];
}

export interface ReportContent {
  sections: ReportSection[];
  generatedAt: string;
  wordCount: number;
  pageEstimate: number;
}

export interface ReportSection {
  id: string;
  title: string;
  order: number;
  content: ContentBlock[];
}

export type ContentBlock =
  | { type: "Paragraph"; text: string }
  | { type: "Heading"; level: number; text: string }
  | { type: "BulletList"; items: string[] }
  | { type: "NumberedList"; items: string[] }
  | { type: "Table"; data: TableData }
  | { type: "Chart"; data: ChartData }
  | { type: "Finding"; data: FindingData }
  | { type: "Metric"; data: MetricData }
  | { type: "Callout"; calloutType: CalloutType; title: string; content: string }
  | { type: "KeyValue"; pairs: Record<string, string> }
  | { type: "PageBreak" }
  | { type: "Spacer"; height: number };

export interface TableData {
  headers: string[];
  rows: string[][];
  caption: string | null;
}

export interface ChartData {
  chartType: ChartType;
  title: string;
  labels: string[];
  datasets: ChartDataset[];
}

export type ChartType = "Bar" | "Line" | "Pie" | "Doughnut" | "Radar";

export interface ChartDataset {
  label: string;
  data: number[];
  color: string | null;
}

export interface FindingData {
  id: string;
  title: string;
  severity: string;
  description: string;
  impact: string;
  recommendation: string;
  affectedAssets: string[];
  references: string[];
}

export interface MetricData {
  label: string;
  value: string;
  change: number | null;
  trend: "Up" | "Down" | "Stable" | null;
  color: string | null;
}

export type CalloutType = "Info" | "Warning" | "Critical" | "Success" | "Note";

export interface Report {
  id: string;
  clientId: string;
  config: ReportConfig;
  status: ReportStatus;
  content: ReportContent | null;
  createdAt: string;
  updatedAt: string;
  generatedAt: string | null;
  fileSize: number | null;
  filePath: string | null;
  error: string | null;
}

export interface ReportSummary {
  id: string;
  title: string;
  reportType: ReportType;
  clientName: string;
  status: ReportStatus;
  format: ExportFormat;
  createdAt: string;
  fileSize: number | null;
}

export interface ReportTypeCount {
  reportType: ReportType;
  count: number;
}

export interface ReportStatusCount {
  status: ReportStatus;
  count: number;
}

export interface ReportStats {
  totalReports: number;
  reportsThisMonth: number;
  byType: ReportTypeCount[];
  byStatus: ReportStatusCount[];
  recentReports: ReportSummary[];
}

// ============================================================================
// Agent Script Types (Task A - Core Mechanics)
// ============================================================================

export interface GenerateAgentScriptRequest {
  clientIp: string;
  authToken: string;
  callbackPort?: number;
  useTls?: boolean;
  heartbeatInterval?: number;
}

export interface AgentScriptResponse {
  success: boolean;
  scriptContent: string;
  scriptId: string;
  generatedAt: string;
  warnings: string[];
}

// ============================================================================
// Native TCP Scanner Types (Task B - Core Mechanics)
// ============================================================================

export interface ScanNetworkRequest {
  cidr: string;
  ports?: number[];
  extended?: boolean;
}

export interface ScanNetworkResponse {
  success: boolean;
  hosts: ScannedHost[];
  hostsScanned: number;
  hostsAlive: number;
  portsScanned: number[];
  durationMs: number;
}

export interface ScannedHost {
  ipAddress: string;
  openPorts: ScannedPort[];
  isAlive: boolean;
  hostname: string | null;
}

export interface ScannedPort {
  port: number;
  open: boolean;
  service: string;
}

// ============================================================================
// Error Types
// ============================================================================

export interface ErrorResponse {
  code: string;
  message: string;
  details?: string;
}
