//! The Factory Module
//!
//! "The Factory" is Optio's dynamic script generation engine. Instead of static
//! downloads, it manufactures unique, state-aware PowerShell scripts for each
//! engagement with identity injection and idempotent operations.

use crate::error::{OptioError, OptioResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Configuration for script generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// Client identifier
    pub client_id: String,
    /// Client display name
    pub client_name: String,
    /// Target subnet for the engagement
    pub target_subnet: String,
    /// Consultant's IP address for mutual auth
    pub consultant_ip: String,
    /// Enable WinRM on target
    pub enable_winrm: bool,
    /// Configure DNS settings
    pub configure_dns: bool,
    /// DNS servers to configure
    pub dns_servers: Vec<String>,
    /// Install security agent
    pub install_agent: bool,
    /// Agent installer path/URL
    pub agent_installer: Option<String>,
    /// Enable Windows Firewall logging
    pub enable_firewall_logging: bool,
    /// Custom PowerShell commands
    pub custom_commands: Vec<String>,
}

/// Result of script generation
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedScript {
    /// Unique identifier for this generated script
    pub script_id: String,
    /// The generated script content
    pub content: String,
    /// When the script was generated
    pub generated_at: DateTime<Utc>,
    /// Warnings or notes about the generation
    pub warnings: Vec<String>,
}

/// Information about an available template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInfo {
    /// Template name/identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Template category
    pub category: String,
    /// Required variables
    pub required_vars: Vec<String>,
    /// Template file path
    pub path: String,
}

/// Script generator engine
pub struct ScriptGenerator {
    templates_dir: PathBuf,
}

impl ScriptGenerator {
    /// Create a new script generator
    pub fn new(templates_dir: PathBuf) -> Self {
        ScriptGenerator { templates_dir }
    }

    /// Generate a script from a template with the given configuration
    pub fn generate(&self, template_name: &str, config: &ScriptConfig) -> OptioResult<GeneratedScript> {
        let template_path = self.templates_dir.join(format!("{}.ps1", template_name));

        // Read the template file
        let template_content = if template_path.exists() {
            std::fs::read_to_string(&template_path)?
        } else {
            // Use embedded default template
            get_default_template(template_name)?
        };

        // Build variable map for substitution
        let mut vars = HashMap::new();
        vars.insert("CLIENT_ID", config.client_id.clone());
        vars.insert("CLIENT_NAME", config.client_name.clone());
        vars.insert("TARGET_SUBNET", config.target_subnet.clone());
        vars.insert("CONSULTANT_IP", config.consultant_ip.clone());
        vars.insert("SCRIPT_ID", Uuid::new_v4().to_string());
        vars.insert("GENERATED_AT", Utc::now().to_rfc3339());
        vars.insert("ENABLE_WINRM", config.enable_winrm.to_string());
        vars.insert("CONFIGURE_DNS", config.configure_dns.to_string());
        vars.insert("DNS_SERVERS", config.dns_servers.join(","));
        vars.insert("INSTALL_AGENT", config.install_agent.to_string());
        vars.insert("AGENT_INSTALLER", config.agent_installer.clone().unwrap_or_default());
        vars.insert("ENABLE_FIREWALL_LOGGING", config.enable_firewall_logging.to_string());

        // Build custom commands section
        let custom_section = if config.custom_commands.is_empty() {
            "# No custom commands configured".to_string()
        } else {
            config.custom_commands.iter()
                .map(|cmd| format!("# Custom command\n{}", cmd))
                .collect::<Vec<_>>()
                .join("\n\n")
        };
        vars.insert("CUSTOM_COMMANDS", custom_section);

        // Perform template substitution
        let mut content = template_content;
        for (key, value) in &vars {
            let placeholder = format!("{{{{{}}}}}", key);
            content = content.replace(&placeholder, value);
        }

        // Generate warnings
        let mut warnings = Vec::new();
        if config.enable_winrm {
            warnings.push("WinRM will be enabled - ensure this is authorized for the target environment.".to_string());
        }
        if !config.custom_commands.is_empty() {
            warnings.push(format!("{} custom command(s) will be executed.", config.custom_commands.len()));
        }

        Ok(GeneratedScript {
            script_id: vars.get("SCRIPT_ID").unwrap().clone(),
            content,
            generated_at: Utc::now(),
            warnings,
        })
    }

    /// List all available templates
    pub fn list_templates(&self) -> OptioResult<Vec<TemplateInfo>> {
        let mut templates = vec![
            // Built-in templates
            TemplateInfo {
                name: "smart_prep".to_string(),
                description: "Comprehensive client preparation script with state auditing".to_string(),
                category: "Provisioning".to_string(),
                required_vars: vec!["CLIENT_NAME".to_string(), "TARGET_SUBNET".to_string()],
                path: "smart_prep.ps1".to_string(),
            },
            TemplateInfo {
                name: "winrm_setup".to_string(),
                description: "WinRM configuration for remote management".to_string(),
                category: "Remote Management".to_string(),
                required_vars: vec!["CONSULTANT_IP".to_string()],
                path: "winrm_setup.ps1".to_string(),
            },
            TemplateInfo {
                name: "security_baseline".to_string(),
                description: "Apply security baseline configurations".to_string(),
                category: "Security".to_string(),
                required_vars: vec!["CLIENT_NAME".to_string()],
                path: "security_baseline.ps1".to_string(),
            },
            TemplateInfo {
                name: "agent_deploy".to_string(),
                description: "Deploy security monitoring agent".to_string(),
                category: "Agents".to_string(),
                required_vars: vec!["AGENT_INSTALLER".to_string()],
                path: "agent_deploy.ps1".to_string(),
            },
        ];

        // Add any custom templates from the templates directory
        if self.templates_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.templates_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "ps1") {
                        let name = path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();

                        // Skip if it's a built-in template
                        if templates.iter().any(|t| t.name == name) {
                            continue;
                        }

                        templates.push(TemplateInfo {
                            name: name.clone(),
                            description: format!("Custom template: {}", name),
                            category: "Custom".to_string(),
                            required_vars: vec![],
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }

        Ok(templates)
    }
}

/// Get the default embedded template content
fn get_default_template(name: &str) -> OptioResult<String> {
    match name {
        "smart_prep" => Ok(SMART_PREP_TEMPLATE.to_string()),
        "winrm_setup" => Ok(WINRM_SETUP_TEMPLATE.to_string()),
        "security_baseline" => Ok(SECURITY_BASELINE_TEMPLATE.to_string()),
        "agent_deploy" => Ok(AGENT_DEPLOY_TEMPLATE.to_string()),
        _ => Err(OptioError::TemplateNotFound(name.to_string())),
    }
}

/// Smart Prep template - comprehensive client preparation
const SMART_PREP_TEMPLATE: &str = r#"<#
.SYNOPSIS
    Optio Smart Prep Script - Client Provisioning
.DESCRIPTION
    Dynamically generated by Optio Factory for client: {{CLIENT_NAME}}
    Script ID: {{SCRIPT_ID}}
    Generated: {{GENERATED_AT}}

    This script performs idempotent configuration of target systems.
    All changes are audited before application.
.NOTES
    Consultant IP: {{CONSULTANT_IP}}
    Target Subnet: {{TARGET_SUBNET}}
#>

#Requires -RunAsAdministrator

param(
    [switch]$WhatIf,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$Script:LogPath = "$env:TEMP\optio_smart_prep_$(Get-Date -Format 'yyyyMMdd_HHmmss').log"

# Logging function
function Write-OptioLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    Add-Content -Path $Script:LogPath -Value $logEntry
    switch ($Level) {
        "ERROR" { Write-Host $logEntry -ForegroundColor Red }
        "WARN"  { Write-Host $logEntry -ForegroundColor Yellow }
        "OK"    { Write-Host $logEntry -ForegroundColor Green }
        default { Write-Host $logEntry }
    }
}

# State check functions for idempotency
function Test-WinRMEnabled {
    try {
        $service = Get-Service WinRM -ErrorAction SilentlyContinue
        return ($service -and $service.Status -eq 'Running')
    } catch {
        return $false
    }
}

function Test-FirewallLoggingEnabled {
    try {
        $profile = Get-NetFirewallProfile -Profile Domain,Private,Public -ErrorAction SilentlyContinue
        return ($profile | Where-Object { $_.LogAllowed -eq $true -or $_.LogBlocked -eq $true })
    } catch {
        return $false
    }
}

# Main execution
Write-OptioLog "========================================" "INFO"
Write-OptioLog "Optio Smart Prep - Starting" "INFO"
Write-OptioLog "Client: {{CLIENT_NAME}}" "INFO"
Write-OptioLog "Target Subnet: {{TARGET_SUBNET}}" "INFO"
Write-OptioLog "Consultant IP: {{CONSULTANT_IP}}" "INFO"
Write-OptioLog "========================================" "INFO"

# WinRM Configuration
if ("{{ENABLE_WINRM}}" -eq "true") {
    Write-OptioLog "Checking WinRM status..." "INFO"
    if (Test-WinRMEnabled) {
        Write-OptioLog "WinRM is already enabled and running" "OK"
    } else {
        if (-not $WhatIf) {
            Write-OptioLog "Enabling WinRM..." "INFO"
            Enable-PSRemoting -Force -SkipNetworkProfileCheck

            # Configure trusted hosts for consultant IP
            Set-Item WSMan:\localhost\Client\TrustedHosts -Value "{{CONSULTANT_IP}}" -Force

            Write-OptioLog "WinRM enabled successfully" "OK"
        } else {
            Write-OptioLog "[WhatIf] Would enable WinRM" "WARN"
        }
    }
}

# DNS Configuration
if ("{{CONFIGURE_DNS}}" -eq "true") {
    Write-OptioLog "Configuring DNS servers..." "INFO"
    $dnsServers = "{{DNS_SERVERS}}" -split ","
    if ($dnsServers.Count -gt 0 -and $dnsServers[0] -ne "") {
        if (-not $WhatIf) {
            $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }
            foreach ($adapter in $adapters) {
                Set-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -ServerAddresses $dnsServers
                Write-OptioLog "Set DNS on adapter: $($adapter.Name)" "OK"
            }
        } else {
            Write-OptioLog "[WhatIf] Would set DNS servers: $($dnsServers -join ', ')" "WARN"
        }
    }
}

# Firewall Logging
if ("{{ENABLE_FIREWALL_LOGGING}}" -eq "true") {
    Write-OptioLog "Configuring firewall logging..." "INFO"
    if (-not $WhatIf) {
        Set-NetFirewallProfile -Profile Domain,Private,Public -LogAllowed True -LogBlocked True -LogFileName "%SystemRoot%\System32\LogFiles\Firewall\pfirewall.log"
        Write-OptioLog "Firewall logging enabled" "OK"
    } else {
        Write-OptioLog "[WhatIf] Would enable firewall logging" "WARN"
    }
}

# Agent Installation
if ("{{INSTALL_AGENT}}" -eq "true") {
    $installerPath = "{{AGENT_INSTALLER}}"
    if ($installerPath -ne "") {
        Write-OptioLog "Installing security agent from: $installerPath" "INFO"
        if (-not $WhatIf) {
            if ($installerPath -match "^https?://") {
                $localPath = "$env:TEMP\agent_installer.exe"
                Invoke-WebRequest -Uri $installerPath -OutFile $localPath -UseBasicParsing
                Start-Process -FilePath $localPath -ArgumentList "/quiet" -Wait
                Remove-Item $localPath -Force
            } else {
                Start-Process -FilePath $installerPath -ArgumentList "/quiet" -Wait
            }
            Write-OptioLog "Agent installed successfully" "OK"
        } else {
            Write-OptioLog "[WhatIf] Would install agent from: $installerPath" "WARN"
        }
    }
}

# Custom Commands Section
{{CUSTOM_COMMANDS}}

Write-OptioLog "========================================" "INFO"
Write-OptioLog "Smart Prep completed successfully" "OK"
Write-OptioLog "Log file: $Script:LogPath" "INFO"
Write-OptioLog "========================================" "INFO"
"#;

/// WinRM Setup template
const WINRM_SETUP_TEMPLATE: &str = r#"<#
.SYNOPSIS
    Optio WinRM Configuration Script
.DESCRIPTION
    Generated by Optio Factory
    Script ID: {{SCRIPT_ID}}

    Configures Windows Remote Management for secure consultant access.
#>

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

Write-Host "Configuring WinRM for Optio..." -ForegroundColor Cyan

# Enable WinRM
Enable-PSRemoting -Force -SkipNetworkProfileCheck

# Configure HTTPS listener (if certificate available)
$cert = Get-ChildItem Cert:\LocalMachine\My | Where-Object { $_.Subject -match $env:COMPUTERNAME } | Select-Object -First 1

if ($cert) {
    $thumbprint = $cert.Thumbprint
    New-Item -Path WSMan:\LocalHost\Listener -Transport HTTPS -Address * -CertificateThumbPrint $thumbprint -Force
    Write-Host "HTTPS listener configured with certificate: $thumbprint" -ForegroundColor Green
} else {
    Write-Host "No suitable certificate found. Using HTTP only (not recommended for production)." -ForegroundColor Yellow
}

# Configure trusted hosts
Set-Item WSMan:\localhost\Client\TrustedHosts -Value "{{CONSULTANT_IP}}" -Force

# Configure firewall rules
Enable-NetFirewallRule -DisplayGroup "Windows Remote Management"

Write-Host "WinRM configuration complete." -ForegroundColor Green
Write-Host "Consultant IP trusted: {{CONSULTANT_IP}}" -ForegroundColor Cyan
"#;

/// Security Baseline template
const SECURITY_BASELINE_TEMPLATE: &str = r#"<#
.SYNOPSIS
    Optio Security Baseline Script
.DESCRIPTION
    Generated by Optio Factory for: {{CLIENT_NAME}}
    Script ID: {{SCRIPT_ID}}

    Applies security baseline configurations based on CIS benchmarks.
#>

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

Write-Host "Applying Optio Security Baseline for {{CLIENT_NAME}}..." -ForegroundColor Cyan

# Password Policy
Write-Host "Configuring password policy..." -ForegroundColor Yellow
net accounts /minpwlen:14 /maxpwage:90 /minpwage:1 /uniquepw:24

# Audit Policy
Write-Host "Configuring audit policy..." -ForegroundColor Yellow
auditpol /set /category:"Logon/Logoff" /success:enable /failure:enable
auditpol /set /category:"Account Management" /success:enable /failure:enable
auditpol /set /category:"Policy Change" /success:enable /failure:enable

# Disable SMBv1
Write-Host "Disabling SMBv1..." -ForegroundColor Yellow
Set-SmbServerConfiguration -EnableSMB1Protocol $false -Force

# Enable Windows Defender features
Write-Host "Configuring Windows Defender..." -ForegroundColor Yellow
Set-MpPreference -DisableRealtimeMonitoring $false
Set-MpPreference -MAPSReporting Advanced
Set-MpPreference -SubmitSamplesConsent SendSafeSamples

# Configure Windows Update
Write-Host "Configuring Windows Update..." -ForegroundColor Yellow
$AutoUpdate = (New-Object -ComObject Microsoft.Update.AutoUpdate)
$AutoUpdate.EnableService()

Write-Host "Security baseline applied successfully." -ForegroundColor Green
"#;

/// Agent Deploy template
const AGENT_DEPLOY_TEMPLATE: &str = r#"<#
.SYNOPSIS
    Optio Agent Deployment Script
.DESCRIPTION
    Generated by Optio Factory
    Script ID: {{SCRIPT_ID}}

    Deploys security monitoring agent to target system.
#>

#Requires -RunAsAdministrator

param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"
$InstallerPath = "{{AGENT_INSTALLER}}"

if ($Uninstall) {
    Write-Host "Uninstalling agent..." -ForegroundColor Yellow
    # Add uninstall logic here based on agent type
    exit 0
}

Write-Host "Deploying security agent for {{CLIENT_NAME}}..." -ForegroundColor Cyan
Write-Host "Installer: $InstallerPath" -ForegroundColor Gray

if ($InstallerPath -eq "") {
    Write-Error "No agent installer specified."
    exit 1
}

# Download if URL
if ($InstallerPath -match "^https?://") {
    $localPath = "$env:TEMP\agent_installer.exe"
    Write-Host "Downloading from: $InstallerPath" -ForegroundColor Yellow

    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $InstallerPath -OutFile $localPath -UseBasicParsing

    $InstallerPath = $localPath
}

# Verify file exists
if (-not (Test-Path $InstallerPath)) {
    Write-Error "Installer not found: $InstallerPath"
    exit 1
}

# Install
Write-Host "Installing agent..." -ForegroundColor Yellow
$process = Start-Process -FilePath $InstallerPath -ArgumentList "/quiet", "/norestart" -Wait -PassThru

if ($process.ExitCode -eq 0) {
    Write-Host "Agent installed successfully." -ForegroundColor Green
} else {
    Write-Error "Installation failed with exit code: $($process.ExitCode)"
    exit 1
}

# Cleanup
if ($InstallerPath -like "$env:TEMP\*") {
    Remove-Item $InstallerPath -Force -ErrorAction SilentlyContinue
}
"#;

/// Agent script configuration for reverse connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentScriptConfig {
    /// IP address of the Optio server (callback target)
    pub client_ip: String,
    /// Authentication token for secure communication
    pub auth_token: String,
    /// Callback port (default: 443)
    pub callback_port: u16,
    /// Enable TLS for callback connection
    pub use_tls: bool,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u32,
}

impl Default for AgentScriptConfig {
    fn default() -> Self {
        Self {
            client_ip: String::new(),
            auth_token: String::new(),
            callback_port: 443,
            use_tls: true,
            heartbeat_interval: 30,
        }
    }
}

/// Generate an agent script with hardcoded connection parameters
pub fn generate_agent_script(config: &AgentScriptConfig) -> OptioResult<GeneratedScript> {
    // Validate configuration
    if config.client_ip.is_empty() {
        return Err(OptioError::Validation("Client IP is required".to_string()));
    }
    if config.auth_token.is_empty() {
        return Err(OptioError::Validation("Auth token is required".to_string()));
    }

    // Build variable map for substitution
    let mut vars = HashMap::new();
    vars.insert("CLIENT_IP", config.client_ip.clone());
    vars.insert("AUTH_TOKEN", config.auth_token.clone());
    vars.insert("CALLBACK_PORT", config.callback_port.to_string());
    vars.insert("USE_TLS", if config.use_tls { "true" } else { "false" }.to_string());
    vars.insert("HEARTBEAT_INTERVAL", config.heartbeat_interval.to_string());
    vars.insert("SCRIPT_ID", Uuid::new_v4().to_string());
    vars.insert("GENERATED_AT", Utc::now().to_rfc3339());

    // Perform template substitution
    let mut content = AGENT_CALLBACK_TEMPLATE.to_string();
    for (key, value) in &vars {
        let placeholder = format!("{{{{{}}}}}", key);
        content = content.replace(&placeholder, value);
    }

    // Generate warnings
    let mut warnings = Vec::new();
    if !config.use_tls {
        warnings.push("TLS is disabled - connection will not be encrypted!".to_string());
    }
    if config.callback_port != 443 && config.callback_port != 8443 {
        warnings.push(format!("Non-standard callback port {} may be blocked by firewalls", config.callback_port));
    }

    Ok(GeneratedScript {
        script_id: vars.get("SCRIPT_ID").unwrap().clone(),
        content,
        generated_at: Utc::now(),
        warnings,
    })
}

/// Agent callback template - creates a reverse connection to Optio
const AGENT_CALLBACK_TEMPLATE: &str = r#"<#
.SYNOPSIS
    Optio Agent Callback Script
.DESCRIPTION
    Dynamically generated by Optio Factory
    Script ID: {{SCRIPT_ID}}
    Generated: {{GENERATED_AT}}

    This script establishes a secure callback connection to the Optio server
    for remote management and telemetry collection.
.NOTES
    Callback Target: {{CLIENT_IP}}:{{CALLBACK_PORT}}
    TLS Enabled: {{USE_TLS}}
#>

#Requires -Version 5.1

param(
    [switch]$Install,
    [switch]$Uninstall,
    [switch]$Status
)

$ErrorActionPreference = "Stop"

# Hardcoded configuration - injected by Optio Factory
$script:Config = @{
    ServerIP = "{{CLIENT_IP}}"
    ServerPort = {{CALLBACK_PORT}}
    AuthToken = "{{AUTH_TOKEN}}"
    UseTLS = ${{USE_TLS}}
    HeartbeatInterval = {{HEARTBEAT_INTERVAL}}
    ScriptId = "{{SCRIPT_ID}}"
}

# Logging function
function Write-AgentLog {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"

    # Log to Windows Event Log if running as service
    try {
        if ([System.Diagnostics.EventLog]::SourceExists("OptioAgent")) {
            $eventType = switch ($Level) {
                "ERROR" { "Error" }
                "WARN" { "Warning" }
                default { "Information" }
            }
            Write-EventLog -LogName "Application" -Source "OptioAgent" -EventId 1000 -EntryType $eventType -Message $Message
        }
    } catch {
        # Fallback to console
    }

    switch ($Level) {
        "ERROR" { Write-Host $logEntry -ForegroundColor Red }
        "WARN"  { Write-Host $logEntry -ForegroundColor Yellow }
        "OK"    { Write-Host $logEntry -ForegroundColor Green }
        default { Write-Host $logEntry }
    }
}

# Create secure connection to Optio server
function Connect-OptioServer {
    Write-AgentLog "Initiating callback to $($Config.ServerIP):$($Config.ServerPort)"

    try {
        $tcpClient = New-Object System.Net.Sockets.TcpClient
        $tcpClient.Connect($Config.ServerIP, $Config.ServerPort)

        $stream = $tcpClient.GetStream()

        if ($Config.UseTLS) {
            Write-AgentLog "Establishing TLS connection..."
            $sslStream = New-Object System.Net.Security.SslStream($stream, $false)
            $sslStream.AuthenticateAsClient($Config.ServerIP)
            $stream = $sslStream
        }

        # Send authentication
        $authPayload = @{
            Type = "AUTH"
            Token = $Config.AuthToken
            ScriptId = $Config.ScriptId
            Hostname = $env:COMPUTERNAME
            Username = $env:USERNAME
            OSVersion = [System.Environment]::OSVersion.VersionString
            Timestamp = (Get-Date).ToUniversalTime().ToString("o")
        } | ConvertTo-Json -Compress

        $bytes = [System.Text.Encoding]::UTF8.GetBytes($authPayload + "`n")
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()

        Write-AgentLog "Connection established and authenticated" "OK"

        return @{
            Client = $tcpClient
            Stream = $stream
        }
    } catch {
        Write-AgentLog "Connection failed: $_" "ERROR"
        throw
    }
}

# Send heartbeat to server
function Send-Heartbeat {
    param($Stream)

    $heartbeat = @{
        Type = "HEARTBEAT"
        ScriptId = $Config.ScriptId
        Timestamp = (Get-Date).ToUniversalTime().ToString("o")
        Memory = [Math]::Round((Get-Process -Id $PID).WorkingSet64 / 1MB, 2)
        Uptime = [Math]::Round((Get-Date) - (Get-Process -Id $PID).StartTime.TotalSeconds, 0)
    } | ConvertTo-Json -Compress

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($heartbeat + "`n")
    $Stream.Write($bytes, 0, $bytes.Length)
    $Stream.Flush()
}

# Collect system telemetry
function Get-SystemTelemetry {
    @{
        Hostname = $env:COMPUTERNAME
        Domain = $env:USERDOMAIN
        Username = $env:USERNAME
        OSVersion = [System.Environment]::OSVersion.VersionString
        PSVersion = $PSVersionTable.PSVersion.ToString()
        Is64Bit = [Environment]::Is64BitOperatingSystem
        ProcessorCount = $env:NUMBER_OF_PROCESSORS
        TotalMemoryMB = [Math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1MB, 0)
        IPAddresses = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike "127.*" }).IPAddress
        LastBootTime = (Get-CimInstance Win32_OperatingSystem).LastBootUpTime.ToString("o")
    }
}

# Main agent loop
function Start-AgentLoop {
    Write-AgentLog "Starting Optio Agent (ScriptId: $($Config.ScriptId))"

    $retryCount = 0
    $maxRetries = 5
    $retryDelay = 10

    while ($true) {
        try {
            $connection = Connect-OptioServer
            $retryCount = 0  # Reset on successful connection

            # Send initial telemetry
            $telemetry = @{
                Type = "TELEMETRY"
                ScriptId = $Config.ScriptId
                Data = Get-SystemTelemetry
            } | ConvertTo-Json -Compress -Depth 5

            $bytes = [System.Text.Encoding]::UTF8.GetBytes($telemetry + "`n")
            $connection.Stream.Write($bytes, 0, $bytes.Length)
            $connection.Stream.Flush()

            # Heartbeat loop
            while ($connection.Client.Connected) {
                Start-Sleep -Seconds $Config.HeartbeatInterval
                Send-Heartbeat -Stream $connection.Stream
            }

        } catch {
            Write-AgentLog "Connection lost: $_" "WARN"
            $retryCount++

            if ($retryCount -ge $maxRetries) {
                Write-AgentLog "Max retries exceeded. Exiting." "ERROR"
                exit 1
            }

            $waitTime = $retryDelay * [Math]::Pow(2, $retryCount - 1)
            Write-AgentLog "Retry $retryCount/$maxRetries in $waitTime seconds..."
            Start-Sleep -Seconds $waitTime
        } finally {
            if ($connection -and $connection.Client) {
                $connection.Client.Close()
            }
        }
    }
}

# Handle command-line switches
if ($Status) {
    Write-Host "Optio Agent Configuration:"
    Write-Host "  Server: $($Config.ServerIP):$($Config.ServerPort)"
    Write-Host "  TLS: $($Config.UseTLS)"
    Write-Host "  Script ID: $($Config.ScriptId)"
    exit 0
}

if ($Uninstall) {
    Write-AgentLog "Uninstalling Optio Agent..."
    # Remove scheduled task if exists
    try {
        Unregister-ScheduledTask -TaskName "OptioAgent" -Confirm:$false -ErrorAction SilentlyContinue
    } catch {}
    Write-AgentLog "Uninstall complete" "OK"
    exit 0
}

if ($Install) {
    Write-AgentLog "Installing Optio Agent as scheduled task..."

    $scriptPath = $MyInvocation.MyCommand.Path
    $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$scriptPath`""
    $trigger = New-ScheduledTaskTrigger -AtStartup
    $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)

    Register-ScheduledTask -TaskName "OptioAgent" -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force

    Write-AgentLog "Installation complete" "OK"
    Start-ScheduledTask -TaskName "OptioAgent"
    exit 0
}

# Default: Run agent loop
Start-AgentLoop
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_generation() {
        let generator = ScriptGenerator::new(PathBuf::from("templates"));
        let config = ScriptConfig {
            client_id: "test-123".to_string(),
            client_name: "Test Client".to_string(),
            target_subnet: "192.168.1.0/24".to_string(),
            consultant_ip: "10.0.0.1".to_string(),
            enable_winrm: true,
            configure_dns: false,
            dns_servers: vec![],
            install_agent: false,
            agent_installer: None,
            enable_firewall_logging: true,
            custom_commands: vec![],
        };

        let result = generator.generate("smart_prep", &config).unwrap();

        assert!(!result.script_id.is_empty());
        assert!(result.content.contains("Test Client"));
        assert!(result.content.contains("192.168.1.0/24"));
        assert!(result.content.contains("10.0.0.1"));
    }

    #[test]
    fn test_list_templates() {
        let generator = ScriptGenerator::new(PathBuf::from("templates"));
        let templates = generator.list_templates().unwrap();

        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.name == "smart_prep"));
    }

    #[test]
    fn test_agent_script_generation() {
        let config = AgentScriptConfig {
            client_ip: "192.168.1.100".to_string(),
            auth_token: "secret-token-123".to_string(),
            callback_port: 443,
            use_tls: true,
            heartbeat_interval: 30,
        };

        let result = generate_agent_script(&config).unwrap();

        assert!(!result.script_id.is_empty());
        assert!(result.content.contains("192.168.1.100"));
        assert!(result.content.contains("secret-token-123"));
        assert!(result.content.contains("443"));
        assert!(result.warnings.is_empty()); // Should have no warnings with default port and TLS
    }

    #[test]
    fn test_agent_script_validation() {
        let config = AgentScriptConfig {
            client_ip: "".to_string(), // Missing IP
            auth_token: "token".to_string(),
            ..Default::default()
        };

        let result = generate_agent_script(&config);
        assert!(result.is_err());
    }
}
