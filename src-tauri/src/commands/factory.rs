//! Factory Module Commands
//!
//! "The Factory" - Dynamic client provisioning with PowerShell script generation.
//! Manufactures unique, state-aware scripts for each engagement.

use crate::error::{OptioError, OptioResult};
use crate::factory::{ScriptConfig, ScriptGenerator, TemplateInfo};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use std::path::PathBuf;

/// Request payload for script generation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateScriptRequest {
    /// Client identifier
    pub client_id: String,
    /// Client display name
    pub client_name: String,
    /// Target subnet for the engagement
    pub target_subnet: String,
    /// Template to use for generation
    pub template_name: String,
    /// Configuration options
    pub config: ScriptConfigOptions,
}

/// Configuration options for script generation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptConfigOptions {
    /// Enable WinRM on target
    pub enable_winrm: bool,
    /// Configure DNS settings
    pub configure_dns: bool,
    /// DNS servers to configure
    pub dns_servers: Option<Vec<String>>,
    /// Install security agent
    pub install_agent: bool,
    /// Agent installer URL or path
    pub agent_installer: Option<String>,
    /// Enable Windows Firewall logging
    pub enable_firewall_logging: bool,
    /// Custom PowerShell commands to include
    pub custom_commands: Option<Vec<String>>,
}

/// Response from script generation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateScriptResponse {
    /// Whether generation was successful
    pub success: bool,
    /// Path to the generated script
    pub output_path: String,
    /// Script content for preview
    pub script_content: String,
    /// Unique script identifier
    pub script_id: String,
    /// Generation timestamp
    pub generated_at: String,
    /// Warnings or notes
    pub warnings: Vec<String>,
}

/// Generate a client provisioning script
#[tauri::command]
pub async fn generate_client_script(
    app_handle: AppHandle,
    request: GenerateScriptRequest,
) -> Result<GenerateScriptResponse, String> {
    tracing::info!(
        "Generating script for client: {} ({})",
        request.client_name,
        request.client_id
    );

    // Get the consultant's IP for injection
    let consultant_ip = crate::commands::system::detect_local_ip()
        .unwrap_or_else(|| "127.0.0.1".to_string());

    // Build the script configuration
    let config = ScriptConfig {
        client_id: request.client_id.clone(),
        client_name: request.client_name.clone(),
        target_subnet: request.target_subnet.clone(),
        consultant_ip,
        enable_winrm: request.config.enable_winrm,
        configure_dns: request.config.configure_dns,
        dns_servers: request.config.dns_servers.unwrap_or_default(),
        install_agent: request.config.install_agent,
        agent_installer: request.config.agent_installer,
        enable_firewall_logging: request.config.enable_firewall_logging,
        custom_commands: request.config.custom_commands.unwrap_or_default(),
    };

    // Get the templates directory
    let templates_dir = get_templates_dir(&app_handle)?;

    // Generate the script
    let generator = ScriptGenerator::new(templates_dir);
    let result = generator
        .generate(&request.template_name, &config)
        .map_err(|e| e.to_string())?;

    // Write to the output directory
    let output_dir = get_output_dir(&app_handle)?;
    let output_filename = format!(
        "{}_{}.ps1",
        sanitize_filename(&request.client_name),
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let output_path = output_dir.join(&output_filename);

    std::fs::write(&output_path, &result.content).map_err(|e| e.to_string())?;

    tracing::info!("Script generated: {:?}", output_path);

    Ok(GenerateScriptResponse {
        success: true,
        output_path: output_path.to_string_lossy().to_string(),
        script_content: result.content,
        script_id: result.script_id,
        generated_at: result.generated_at.to_rfc3339(),
        warnings: result.warnings,
    })
}

/// List available script templates
#[tauri::command]
pub async fn list_templates(app_handle: AppHandle) -> Result<Vec<TemplateInfo>, String> {
    let templates_dir = get_templates_dir(&app_handle)?;
    let generator = ScriptGenerator::new(templates_dir);
    generator.list_templates().map_err(|e| e.to_string())
}

/// Preview script generation request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRequest {
    pub template_name: String,
    pub config: ScriptConfigOptions,
    pub client_name: String,
    pub target_subnet: String,
}

/// Get a preview of the generated script without saving
#[tauri::command]
pub async fn get_script_preview(
    app_handle: AppHandle,
    request: PreviewRequest,
) -> Result<String, String> {
    let consultant_ip = crate::commands::system::detect_local_ip()
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let config = ScriptConfig {
        client_id: "preview".to_string(),
        client_name: request.client_name,
        target_subnet: request.target_subnet,
        consultant_ip,
        enable_winrm: request.config.enable_winrm,
        configure_dns: request.config.configure_dns,
        dns_servers: request.config.dns_servers.unwrap_or_default(),
        install_agent: request.config.install_agent,
        agent_installer: request.config.agent_installer,
        enable_firewall_logging: request.config.enable_firewall_logging,
        custom_commands: request.config.custom_commands.unwrap_or_default(),
    };

    let templates_dir = get_templates_dir(&app_handle)?;
    let generator = ScriptGenerator::new(templates_dir);
    let result = generator
        .generate(&request.template_name, &config)
        .map_err(|e| e.to_string())?;

    Ok(result.content)
}

/// Validation request for configuration
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateConfigRequest {
    pub client_name: String,
    pub target_subnet: String,
    pub config: ScriptConfigOptions,
}

/// Validation result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate script configuration before generation
#[tauri::command]
pub async fn validate_config(request: ValidateConfigRequest) -> Result<ValidationResult, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate client name
    if request.client_name.trim().is_empty() {
        errors.push("Client name is required".to_string());
    }

    // Validate target subnet
    if request.target_subnet.trim().is_empty() {
        errors.push("Target subnet is required".to_string());
    } else if !is_valid_subnet(&request.target_subnet) {
        errors.push("Invalid subnet format. Expected format: 192.168.1.0/24".to_string());
    }

    // Validate DNS servers if configured
    if request.config.configure_dns {
        if let Some(ref servers) = request.config.dns_servers {
            if servers.is_empty() {
                errors.push("DNS servers must be specified when DNS configuration is enabled".to_string());
            } else {
                for server in servers {
                    if !is_valid_ip(server) {
                        errors.push(format!("Invalid DNS server IP: {}", server));
                    }
                }
            }
        } else {
            errors.push("DNS servers must be specified when DNS configuration is enabled".to_string());
        }
    }

    // Validate agent installer if enabled
    if request.config.install_agent {
        if request.config.agent_installer.as_ref().map_or(true, |s| s.trim().is_empty()) {
            errors.push("Agent installer path/URL is required when agent installation is enabled".to_string());
        }
    }

    // Add warnings for potentially risky configurations
    if request.config.enable_winrm {
        warnings.push("WinRM enablement will modify Windows Remote Management settings".to_string());
    }

    if !request.config.custom_commands.as_ref().map_or(true, |c| c.is_empty()) {
        warnings.push("Custom commands will be executed. Review them carefully before deployment.".to_string());
    }

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}

// Helper functions

fn get_templates_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    // In development, use the local templates directory
    // In production, use the bundled resources
    let resource_path = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let templates_path = resource_path.join("templates");

    if templates_path.exists() {
        Ok(templates_path)
    } else {
        // Fallback to local development path
        Ok(PathBuf::from("templates"))
    }
}

fn get_output_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let output_dir = app_data.join("generated_scripts");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    Ok(output_dir)
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn is_valid_subnet(subnet: &str) -> bool {
    let parts: Vec<&str> = subnet.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    // Check CIDR notation
    if let Ok(prefix) = parts[1].parse::<u8>() {
        if prefix > 32 {
            return false;
        }
    } else {
        return false;
    }

    // Check IP address
    is_valid_ip(parts[0])
}

fn is_valid_ip(ip: &str) -> bool {
    let octets: Vec<&str> = ip.split('.').collect();
    if octets.len() != 4 {
        return false;
    }

    octets.iter().all(|octet| {
        octet.parse::<u8>().is_ok()
    })
}
