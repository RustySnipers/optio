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
