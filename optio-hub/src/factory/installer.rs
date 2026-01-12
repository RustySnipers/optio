//! Agent Installer Generator (The Factory)
//!
//! Generates deployment packages for optio-agent including:
//! - Agent binary
//! - mTLS certificates (CA, client cert/key)
//! - Configuration file (config.toml)
//! - Installation PowerShell script

use crate::error::{OptioError, OptioResult};
use chrono::{DateTime, Utc};
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose,
    IsCa, KeyPair, KeyUsagePurpose,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Configuration for installer generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerConfig {
    /// Client name for identification
    pub client_name: String,
    /// Hub server address (e.g., "192.168.1.100:50051")
    pub hub_address: String,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u32,
    /// Custom agent ID (if not provided, auto-generated)
    pub agent_id: Option<String>,
}

impl Default for InstallerConfig {
    fn default() -> Self {
        Self {
            client_name: "default".to_string(),
            hub_address: "127.0.0.1:50051".to_string(),
            heartbeat_interval: 30,
            agent_id: None,
        }
    }
}

/// Result of installer generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerResult {
    /// Unique agent identifier
    pub agent_id: String,
    /// Path to the generated ZIP file
    pub installer_path: String,
    /// Size of the ZIP file in bytes
    pub file_size: u64,
    /// When the installer was generated
    pub generated_at: DateTime<Utc>,
    /// Warnings during generation
    pub warnings: Vec<String>,
}

/// Generated certificate bundle
struct CertBundle {
    ca_cert_pem: String,
    client_cert_pem: String,
    client_key_pem: String,
}

/// The Installer Generator
pub struct InstallerGenerator {
    /// Base directory for output
    output_dir: PathBuf,
    /// Path to CA certificate (existing)
    ca_cert_path: Option<PathBuf>,
    /// Path to CA private key (for signing)
    ca_key_path: Option<PathBuf>,
    /// Path to agent binary
    agent_binary_path: Option<PathBuf>,
}

impl InstallerGenerator {
    /// Create a new installer generator
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            ca_cert_path: None,
            ca_key_path: None,
            agent_binary_path: None,
        }
    }

    /// Set the CA certificate and key paths for signing client certs
    pub fn with_ca(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.ca_cert_path = Some(cert_path);
        self.ca_key_path = Some(key_path);
        self
    }

    /// Set the agent binary path
    pub fn with_agent_binary(mut self, path: PathBuf) -> Self {
        self.agent_binary_path = Some(path);
        self
    }

    /// Generate a complete installer package
    pub fn generate(&self, config: &InstallerConfig) -> OptioResult<InstallerResult> {
        let mut warnings = Vec::new();

        // Generate agent ID if not provided
        let agent_id = config.agent_id.clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create temp directory for package contents
        let package_dir = self.output_dir.join(format!("optio-agent-{}", &agent_id[..8]));
        fs::create_dir_all(&package_dir)?;

        // Step 1: Generate or load certificates
        let certs = self.generate_client_certificates(&agent_id, &config.client_name)?;

        // Step 2: Write certificate files
        self.write_certificates(&package_dir, &certs)?;

        // Step 3: Generate config.toml
        self.write_config(&package_dir, config, &agent_id)?;

        // Step 4: Generate install.ps1
        self.write_install_script(&package_dir, &agent_id)?;

        // Step 5: Copy agent binary (if available)
        if let Some(ref binary_path) = self.agent_binary_path {
            if binary_path.exists() {
                let dest = package_dir.join("optio-agent.exe");
                fs::copy(binary_path, &dest)?;
                tracing::info!("Copied agent binary to package");
            } else {
                warnings.push(format!(
                    "Agent binary not found at {}. Package will not include executable.",
                    binary_path.display()
                ));
            }
        } else {
            // Try default locations
            let default_paths = [
                PathBuf::from("target/release/optio-agent.exe"),
                PathBuf::from("target/release/optio-agent"),
                PathBuf::from("../optio-agent/target/release/optio-agent.exe"),
                PathBuf::from("../optio-agent/target/release/optio-agent"),
            ];

            let mut found = false;
            for path in &default_paths {
                if path.exists() {
                    let dest_name = if cfg!(windows) {
                        "optio-agent.exe"
                    } else {
                        "optio-agent"
                    };
                    fs::copy(path, package_dir.join(dest_name))?;
                    tracing::info!("Found and copied agent binary from {}", path.display());
                    found = true;
                    break;
                }
            }

            if !found {
                warnings.push(
                    "Agent binary not found. Build with 'cargo build --release -p optio-agent' first.".to_string()
                );
            }
        }

        // Step 6: Create ZIP archive
        let zip_filename = format!("optio-agent-{}-{}.zip",
            sanitize_filename(&config.client_name),
            &agent_id[..8]
        );
        let zip_path = self.output_dir.join(&zip_filename);
        let file_size = self.create_zip_archive(&package_dir, &zip_path)?;

        // Cleanup temp directory
        fs::remove_dir_all(&package_dir)?;

        tracing::info!(
            agent_id = %agent_id,
            zip_path = %zip_path.display(),
            file_size = file_size,
            "Generated agent installer package"
        );

        Ok(InstallerResult {
            agent_id,
            installer_path: zip_path.to_string_lossy().to_string(),
            file_size,
            generated_at: Utc::now(),
            warnings,
        })
    }

    /// Generate client certificates signed by the CA
    fn generate_client_certificates(
        &self,
        agent_id: &str,
        client_name: &str,
    ) -> OptioResult<CertBundle> {
        // If we have existing CA, load and use it
        if let (Some(ca_cert_path), Some(ca_key_path)) = (&self.ca_cert_path, &self.ca_key_path) {
            return self.sign_with_existing_ca(ca_cert_path, ca_key_path, agent_id, client_name);
        }

        // Otherwise generate a new CA and client cert
        self.generate_new_ca_and_client(agent_id, client_name)
    }

    /// Sign a client cert with an existing CA
    fn sign_with_existing_ca(
        &self,
        ca_cert_path: &Path,
        ca_key_path: &Path,
        agent_id: &str,
        client_name: &str,
    ) -> OptioResult<CertBundle> {
        // Read existing CA
        let ca_cert_pem = fs::read_to_string(ca_cert_path)
            .map_err(|e| OptioError::Validation(format!("Failed to read CA cert: {}", e)))?;
        let ca_key_pem = fs::read_to_string(ca_key_path)
            .map_err(|e| OptioError::Validation(format!("Failed to read CA key: {}", e)))?;

        // Parse CA key
        let ca_key_pair = KeyPair::from_pem(&ca_key_pem)
            .map_err(|e| OptioError::Validation(format!("Invalid CA key: {}", e)))?;

        // Parse CA certificate parameters to sign with
        let ca_params = CertificateParams::from_ca_cert_pem(&ca_cert_pem)
            .map_err(|e| OptioError::Validation(format!("Invalid CA cert: {}", e)))?;
        let ca_cert = ca_params.self_signed(&ca_key_pair)
            .map_err(|e| OptioError::Validation(format!("Failed to parse CA: {}", e)))?;

        // Generate client certificate
        let client_key_pair = KeyPair::generate()
            .map_err(|e| OptioError::Validation(format!("Failed to generate key: {}", e)))?;

        let mut client_params = CertificateParams::new(vec![
            format!("optio-agent-{}", &agent_id[..8]),
            agent_id.to_string(),
        ])
        .map_err(|e| OptioError::Validation(format!("Invalid cert params: {}", e)))?;

        client_params.is_ca = IsCa::NoCa;
        client_params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        client_params.distinguished_name.push(DnType::CommonName, format!("Optio Agent {}", &agent_id[..8]));
        client_params.distinguished_name.push(DnType::OrganizationName, client_name);

        let client_cert = client_params.signed_by(&client_key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| OptioError::Validation(format!("Failed to sign client cert: {}", e)))?;

        Ok(CertBundle {
            ca_cert_pem,
            client_cert_pem: client_cert.pem(),
            client_key_pem: client_key_pair.serialize_pem(),
        })
    }

    /// Generate a new CA and client certificate
    fn generate_new_ca_and_client(
        &self,
        agent_id: &str,
        client_name: &str,
    ) -> OptioResult<CertBundle> {
        // Generate CA
        let ca_key_pair = KeyPair::generate()
            .map_err(|e| OptioError::Validation(format!("Failed to generate CA key: {}", e)))?;

        let mut ca_params = CertificateParams::new(vec!["Optio CA".to_string()])
            .map_err(|e| OptioError::Validation(format!("Invalid CA params: {}", e)))?;

        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
        ];
        ca_params.distinguished_name.push(DnType::CommonName, "Optio CA");
        ca_params.distinguished_name.push(DnType::OrganizationName, client_name);

        let ca_cert = ca_params.self_signed(&ca_key_pair)
            .map_err(|e| OptioError::Validation(format!("Failed to create CA: {}", e)))?;

        // Generate client certificate
        let client_key_pair = KeyPair::generate()
            .map_err(|e| OptioError::Validation(format!("Failed to generate client key: {}", e)))?;

        let mut client_params = CertificateParams::new(vec![
            format!("optio-agent-{}", &agent_id[..8]),
            agent_id.to_string(),
        ])
        .map_err(|e| OptioError::Validation(format!("Invalid cert params: {}", e)))?;

        client_params.is_ca = IsCa::NoCa;
        client_params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        client_params.distinguished_name.push(DnType::CommonName, format!("Optio Agent {}", &agent_id[..8]));
        client_params.distinguished_name.push(DnType::OrganizationName, client_name);

        let client_cert = client_params.signed_by(&client_key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| OptioError::Validation(format!("Failed to sign client cert: {}", e)))?;

        Ok(CertBundle {
            ca_cert_pem: ca_cert.pem(),
            client_cert_pem: client_cert.pem(),
            client_key_pem: client_key_pair.serialize_pem(),
        })
    }

    /// Write certificates to the package directory
    fn write_certificates(&self, package_dir: &Path, certs: &CertBundle) -> OptioResult<()> {
        let certs_dir = package_dir.join("certs");
        fs::create_dir_all(&certs_dir)?;

        fs::write(certs_dir.join("ca.pem"), &certs.ca_cert_pem)?;
        fs::write(certs_dir.join("client.pem"), &certs.client_cert_pem)?;
        fs::write(certs_dir.join("client.key"), &certs.client_key_pem)?;

        Ok(())
    }

    /// Generate and write the config.toml file
    fn write_config(
        &self,
        package_dir: &Path,
        config: &InstallerConfig,
        agent_id: &str,
    ) -> OptioResult<()> {
        let config_content = format!(
            r#"# Optio Agent Configuration
# Generated: {}
# Client: {}

[agent]
# Unique agent identifier
id = "{}"

# Hub server address (include https:// for mTLS)
hub_address = "https://{}"

# Heartbeat interval in seconds
heartbeat_interval = {}

# Command execution timeout in seconds
command_timeout = 300

[tls]
# Path to CA certificate (for server verification)
ca_path = "certs/ca.pem"

# Path to client certificate (for authentication)
cert_path = "certs/client.pem"

# Path to client private key
key_path = "certs/client.key"

[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log to Windows Event Log when running as service
event_log = true
"#,
            Utc::now().to_rfc3339(),
            config.client_name,
            agent_id,
            config.hub_address,
            config.heartbeat_interval,
        );

        fs::write(package_dir.join("config.toml"), config_content)?;
        Ok(())
    }

    /// Generate the install.ps1 script
    fn write_install_script(&self, package_dir: &Path, agent_id: &str) -> OptioResult<()> {
        let script = generate_install_script(agent_id);
        fs::write(package_dir.join("install.ps1"), script)?;
        Ok(())
    }

    /// Create a ZIP archive of the package directory
    fn create_zip_archive(&self, source_dir: &Path, zip_path: &Path) -> OptioResult<u64> {
        let file = File::create(zip_path)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        // Walk the directory and add files
        self.add_directory_to_zip(&mut zip, source_dir, source_dir, options)?;

        zip.finish()?;

        // Get file size
        let metadata = fs::metadata(zip_path)?;
        Ok(metadata.len())
    }

    /// Recursively add directory contents to ZIP
    fn add_directory_to_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        base_path: &Path,
        current_path: &Path,
        options: SimpleFileOptions,
    ) -> OptioResult<()> {
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(base_path)
                .map_err(|e| OptioError::Validation(e.to_string()))?;

            if path.is_dir() {
                // Add directory entry
                let dir_name = format!("{}/", relative_path.to_string_lossy());
                zip.add_directory(&dir_name, options)
                    .map_err(|e| OptioError::Validation(format!("ZIP error: {}", e)))?;

                // Recurse into directory
                self.add_directory_to_zip(zip, base_path, &path, options)?;
            } else {
                // Add file
                let file_name = relative_path.to_string_lossy().replace('\\', "/");
                zip.start_file(&file_name, options)
                    .map_err(|e| OptioError::Validation(format!("ZIP error: {}", e)))?;

                let mut file = File::open(&path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }

        Ok(())
    }
}

/// Generate the PowerShell installation script
fn generate_install_script(agent_id: &str) -> String {
    format!(r#"<#
.SYNOPSIS
    Optio Agent Installer
.DESCRIPTION
    Installs the Optio Agent as a Windows Service.
    Agent ID: {agent_id}

    Run with Administrator privileges.
.PARAMETER Uninstall
    Remove the Optio Agent service and files
.PARAMETER Status
    Show current service status
.EXAMPLE
    .\install.ps1
    Installs the agent as a Windows service
.EXAMPLE
    .\install.ps1 -Uninstall
    Removes the agent service and files
#>

#Requires -RunAsAdministrator

param(
    [switch]$Uninstall,
    [switch]$Status,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# Configuration
$ServiceName = "OptioAgent"
$ServiceDisplayName = "Optio Agent"
$ServiceDescription = "Optio endpoint management agent for secure IT orchestration"
$InstallPath = "$env:ProgramFiles\Optio"
$AgentId = "{agent_id}"

# Logging
function Write-Log {{
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    switch ($Level) {{
        "ERROR" {{ Write-Host $logEntry -ForegroundColor Red }}
        "WARN"  {{ Write-Host $logEntry -ForegroundColor Yellow }}
        "OK"    {{ Write-Host $logEntry -ForegroundColor Green }}
        default {{ Write-Host $logEntry }}
    }}
}}

# Check if service exists
function Test-ServiceExists {{
    param([string]$Name)
    $service = Get-Service -Name $Name -ErrorAction SilentlyContinue
    return $null -ne $service
}}

# Stop and remove service
function Remove-OptioService {{
    Write-Log "Stopping service..."

    if (Test-ServiceExists $ServiceName) {{
        $service = Get-Service -Name $ServiceName
        if ($service.Status -eq 'Running') {{
            Stop-Service -Name $ServiceName -Force
            Write-Log "Service stopped" "OK"
        }}

        Write-Log "Removing service..."
        sc.exe delete $ServiceName | Out-Null
        Start-Sleep -Seconds 2
        Write-Log "Service removed" "OK"
    }} else {{
        Write-Log "Service not found" "WARN"
    }}
}}

# Show status
if ($Status) {{
    Write-Log "=== Optio Agent Status ===" "INFO"
    Write-Log "Install Path: $InstallPath"
    Write-Log "Agent ID: $AgentId"

    if (Test-ServiceExists $ServiceName) {{
        $service = Get-Service -Name $ServiceName
        Write-Log "Service Status: $($service.Status)"
        Write-Log "Startup Type: $($service.StartType)"
    }} else {{
        Write-Log "Service: Not installed" "WARN"
    }}

    if (Test-Path "$InstallPath\optio-agent.exe") {{
        $version = (Get-Item "$InstallPath\optio-agent.exe").VersionInfo.FileVersion
        Write-Log "Agent Version: $version"
    }}

    exit 0
}}

# Uninstall
if ($Uninstall) {{
    Write-Log "=== Uninstalling Optio Agent ===" "INFO"

    Remove-OptioService

    if (Test-Path $InstallPath) {{
        Write-Log "Removing files..."
        Remove-Item -Path $InstallPath -Recurse -Force
        Write-Log "Files removed" "OK"
    }}

    Write-Log "=== Uninstall Complete ===" "OK"
    exit 0
}}

# Install
Write-Log "=== Installing Optio Agent ===" "INFO"
Write-Log "Agent ID: $AgentId"
Write-Log "Install Path: $InstallPath"

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Verify required files
$requiredFiles = @(
    "optio-agent.exe",
    "config.toml",
    "certs\ca.pem",
    "certs\client.pem",
    "certs\client.key"
)

Write-Log "Verifying package contents..."
foreach ($file in $requiredFiles) {{
    $filePath = Join-Path $ScriptDir $file
    if (-not (Test-Path $filePath)) {{
        Write-Log "Missing required file: $file" "ERROR"
        exit 1
    }}
}}
Write-Log "Package verification complete" "OK"

# Stop existing service if running
if (Test-ServiceExists $ServiceName) {{
    if (-not $Force) {{
        $response = Read-Host "Existing installation found. Overwrite? (Y/N)"
        if ($response -notmatch '^[Yy]') {{
            Write-Log "Installation cancelled" "WARN"
            exit 0
        }}
    }}
    Remove-OptioService
}}

# Create installation directory
Write-Log "Creating installation directory..."
if (-not (Test-Path $InstallPath)) {{
    New-Item -Path $InstallPath -ItemType Directory -Force | Out-Null
}}
if (-not (Test-Path "$InstallPath\certs")) {{
    New-Item -Path "$InstallPath\certs" -ItemType Directory -Force | Out-Null
}}

# Copy files
Write-Log "Copying files..."
Copy-Item -Path "$ScriptDir\optio-agent.exe" -Destination "$InstallPath\optio-agent.exe" -Force
Copy-Item -Path "$ScriptDir\config.toml" -Destination "$InstallPath\config.toml" -Force
Copy-Item -Path "$ScriptDir\certs\ca.pem" -Destination "$InstallPath\certs\ca.pem" -Force
Copy-Item -Path "$ScriptDir\certs\client.pem" -Destination "$InstallPath\certs\client.pem" -Force
Copy-Item -Path "$ScriptDir\certs\client.key" -Destination "$InstallPath\certs\client.key" -Force

# Set permissions on private key (restrict access)
Write-Log "Setting file permissions..."
$keyPath = "$InstallPath\certs\client.key"
$acl = Get-Acl $keyPath
$acl.SetAccessRuleProtection($true, $false)
$adminRule = New-Object System.Security.AccessControl.FileSystemAccessRule("SYSTEM", "FullControl", "Allow")
$acl.SetAccessRule($adminRule)
$adminRule2 = New-Object System.Security.AccessControl.FileSystemAccessRule("Administrators", "FullControl", "Allow")
$acl.SetAccessRule($adminRule2)
Set-Acl -Path $keyPath -AclObject $acl

Write-Log "Files copied and secured" "OK"

# Create Windows Service
Write-Log "Creating Windows service..."
$binPath = "`"$InstallPath\optio-agent.exe`" --service --config `"$InstallPath\config.toml`""

$result = sc.exe create $ServiceName `
    binPath= $binPath `
    start= auto `
    DisplayName= $ServiceDisplayName

if ($LASTEXITCODE -ne 0) {{
    Write-Log "Failed to create service: $result" "ERROR"
    exit 1
}}

# Set service description
sc.exe description $ServiceName $ServiceDescription | Out-Null

# Configure service recovery (restart on failure)
sc.exe failure $ServiceName reset= 86400 actions= restart/5000/restart/10000/restart/30000 | Out-Null

Write-Log "Service created" "OK"

# Start the service
Write-Log "Starting service..."
Start-Service -Name $ServiceName
Start-Sleep -Seconds 2

$service = Get-Service -Name $ServiceName
if ($service.Status -eq 'Running') {{
    Write-Log "Service started successfully" "OK"
}} else {{
    Write-Log "Service failed to start. Check Event Log for details." "ERROR"
    exit 1
}}

Write-Log "=== Installation Complete ===" "OK"
Write-Log ""
Write-Log "Agent is now running as a Windows service."
Write-Log "To check status: Get-Service $ServiceName"
Write-Log "To view logs: Get-EventLog -LogName Application -Source OptioAgent"
"#, agent_id = agent_id)
}

/// Sanitize a filename by removing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Acme Corp"), "acme_corp");
        assert_eq!(sanitize_filename("Client-123"), "client-123");
        assert_eq!(sanitize_filename("Test/Client"), "test_client");
    }

    #[test]
    fn test_installer_config_default() {
        let config = InstallerConfig::default();
        assert_eq!(config.heartbeat_interval, 30);
        assert!(config.agent_id.is_none());
    }

    #[test]
    fn test_generate_install_script() {
        let script = generate_install_script("test-agent-id");
        assert!(script.contains("test-agent-id"));
        assert!(script.contains("OptioAgent"));
        assert!(script.contains("Windows service"));
    }
}
