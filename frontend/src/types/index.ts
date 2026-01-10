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
// Error Types
// ============================================================================

export interface ErrorResponse {
  code: string;
  message: string;
  details?: string;
}
