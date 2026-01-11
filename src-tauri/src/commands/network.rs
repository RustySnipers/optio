//! Network Intelligence Commands
//!
//! Tauri commands for network scanning and asset inventory management.

use crate::network::{
    models::*,
    scanner::{
        check_nmap_installed, get_scan_types, build_nmap_command, validate_target,
        get_common_ports, scan_network_native, scan_network_with_ports,
        NmapInfo, ScanTypeInfo, TargetValidation, CommonPort, ScannedHost,
        DEFAULT_SCAN_PORTS, EXTENDED_SCAN_PORTS,
    },
    inventory::{generate_demo_assets, AssetInventory},
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

/// In-memory storage for demo purposes
/// In production, this would be replaced with database persistence
pub struct NetworkState {
    pub inventory: Mutex<AssetInventory>,
    pub scans: Mutex<Vec<ScanJob>>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            inventory: Mutex::new(AssetInventory::new()),
            scans: Mutex::new(Vec::new()),
        }
    }
}

// ============================================================================
// Scanner Commands
// ============================================================================

/// Check if Nmap is installed and get version info
#[tauri::command]
pub async fn check_nmap() -> Result<NmapInfo, String> {
    check_nmap_installed()
}

/// Get available scan types
#[tauri::command]
pub async fn get_scan_type_list() -> Result<Vec<ScanTypeInfo>, String> {
    Ok(get_scan_types())
}

/// Get common ports reference
#[tauri::command]
pub async fn get_common_port_list() -> Result<Vec<CommonPort>, String> {
    Ok(get_common_ports())
}

/// Validate a scan target
#[tauri::command]
pub async fn validate_scan_target(target: String) -> Result<TargetValidation, String> {
    validate_target(&target)
}

/// Create a new scan job request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScanRequest {
    pub client_id: String,
    pub name: String,
    pub targets: Vec<String>,
    pub scan_type: String,
    pub custom_args: Option<String>,
    pub ports: Option<String>,
    pub exclude_targets: Option<Vec<String>>,
    pub aggressive: bool,
    pub skip_discovery: bool,
}

/// Create a new scan job (queued, not executed)
#[tauri::command]
pub async fn create_scan(
    state: State<'_, NetworkState>,
    request: CreateScanRequest,
) -> Result<ScanJob, String> {
    let scan_type = parse_scan_type(&request.scan_type)?;

    let config = ScanConfig {
        targets: request.targets,
        scan_type,
        custom_args: request.custom_args,
        ports: request.ports,
        exclude_targets: request.exclude_targets,
        aggressive: request.aggressive,
        skip_discovery: request.skip_discovery,
        output_formats: vec![OutputFormat::Xml],
    };

    let job = ScanJob {
        id: Uuid::new_v4().to_string(),
        client_id: request.client_id,
        name: request.name,
        config,
        status: ScanStatus::Queued,
        created_at: chrono::Utc::now().to_rfc3339(),
        started_at: None,
        completed_at: None,
        error: None,
        progress: 0,
        raw_output: None,
    };

    let mut scans = state.scans.lock().map_err(|e| e.to_string())?;
    scans.push(job.clone());

    Ok(job)
}

/// Get the Nmap command that would be executed (preview)
#[tauri::command]
pub async fn preview_scan_command(
    targets: Vec<String>,
    scan_type: String,
    ports: Option<String>,
    aggressive: bool,
) -> Result<String, String> {
    let st = parse_scan_type(&scan_type)?;

    let config = ScanConfig {
        targets,
        scan_type: st,
        ports,
        aggressive,
        ..Default::default()
    };

    let args = build_nmap_command(&config);
    Ok(format!("nmap {}", args.join(" ")))
}

/// List all scans for a client
#[tauri::command]
pub async fn list_scans(
    state: State<'_, NetworkState>,
    client_id: String,
) -> Result<Vec<ScanJob>, String> {
    let scans = state.scans.lock().map_err(|e| e.to_string())?;
    Ok(scans.iter()
        .filter(|s| s.client_id == client_id)
        .cloned()
        .collect())
}

/// Get a specific scan by ID
#[tauri::command]
pub async fn get_scan(
    state: State<'_, NetworkState>,
    scan_id: String,
) -> Result<Option<ScanJob>, String> {
    let scans = state.scans.lock().map_err(|e| e.to_string())?;
    Ok(scans.iter().find(|s| s.id == scan_id).cloned())
}

/// Delete a scan
#[tauri::command]
pub async fn delete_scan(
    state: State<'_, NetworkState>,
    scan_id: String,
) -> Result<bool, String> {
    let mut scans = state.scans.lock().map_err(|e| e.to_string())?;
    let len_before = scans.len();
    scans.retain(|s| s.id != scan_id);
    Ok(scans.len() < len_before)
}

// ============================================================================
// Native TCP Scanner Commands (Task B - Core Mechanics)
// ============================================================================

/// Request for native network scan
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanNetworkRequest {
    /// CIDR notation target (e.g., "192.168.1.0/24")
    pub cidr: String,
    /// Optional custom ports to scan (defaults to 22, 80, 443, 3389)
    pub ports: Option<Vec<u16>>,
    /// Use extended port list (24 common ports)
    pub extended: Option<bool>,
}

/// Response from native network scan
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanNetworkResponse {
    /// Whether scan was successful
    pub success: bool,
    /// Discovered hosts with open ports
    pub hosts: Vec<ScannedHost>,
    /// Total hosts scanned
    pub hosts_scanned: usize,
    /// Number of live hosts (with open ports)
    pub hosts_alive: usize,
    /// Ports that were scanned
    pub ports_scanned: Vec<u16>,
    /// Scan duration in milliseconds
    pub duration_ms: u64,
}

/// Scan a network using the native Rust TCP scanner
///
/// This is a lightweight alternative to Nmap that uses async TCP connects
/// to detect open ports on hosts within a CIDR range.
///
/// Default ports: 22 (SSH), 80 (HTTP), 443 (HTTPS), 3389 (RDP)
/// Extended ports: 24 common service ports
#[tauri::command]
pub async fn scan_network(
    request: ScanNetworkRequest,
) -> Result<ScanNetworkResponse, String> {
    tracing::info!("Starting native TCP scan of: {}", request.cidr);

    let start = std::time::Instant::now();

    // Determine which ports to scan
    let ports = if let Some(custom_ports) = request.ports {
        custom_ports
    } else if request.extended.unwrap_or(false) {
        EXTENDED_SCAN_PORTS.to_vec()
    } else {
        DEFAULT_SCAN_PORTS.to_vec()
    };

    // Perform the scan
    let hosts = scan_network_with_ports(&request.cidr, ports.clone()).await?;

    let duration = start.elapsed();

    // Calculate statistics
    // For hosts_scanned, we need to parse the CIDR to get the count
    let hosts_scanned = estimate_host_count(&request.cidr).unwrap_or(0);
    let hosts_alive = hosts.len();

    tracing::info!(
        "Scan complete: {} live hosts found out of ~{} in {} ms",
        hosts_alive,
        hosts_scanned,
        duration.as_millis()
    );

    Ok(ScanNetworkResponse {
        success: true,
        hosts,
        hosts_scanned,
        hosts_alive,
        ports_scanned: ports,
        duration_ms: duration.as_millis() as u64,
    })
}

/// Quick scan of a single host
#[tauri::command]
pub async fn scan_single_host(
    ip: String,
    ports: Option<Vec<u16>>,
) -> Result<ScannedHost, String> {
    tracing::info!("Scanning single host: {}", ip);

    let ports = ports.unwrap_or_else(|| DEFAULT_SCAN_PORTS.to_vec());

    let config = crate::network::scanner::TcpScannerConfig {
        ports,
        ..Default::default()
    };

    let scanner = crate::network::scanner::TcpScanner::with_config(config);
    scanner.scan_single_host(&ip).await
}

/// Get the default ports used for scanning
#[tauri::command]
pub async fn get_default_scan_ports() -> Result<Vec<u16>, String> {
    Ok(DEFAULT_SCAN_PORTS.to_vec())
}

/// Get the extended ports list for thorough scanning
#[tauri::command]
pub async fn get_extended_scan_ports() -> Result<Vec<u16>, String> {
    Ok(EXTENDED_SCAN_PORTS.to_vec())
}

/// Estimate the number of hosts in a CIDR range
fn estimate_host_count(cidr: &str) -> Option<usize> {
    use std::str::FromStr;
    ipnetwork::IpNetwork::from_str(cidr)
        .ok()
        .map(|net| net.size() as usize)
}

// ============================================================================
// Asset Inventory Commands
// ============================================================================

/// Get all assets for a client
#[tauri::command]
pub async fn list_assets(
    state: State<'_, NetworkState>,
    client_id: String,
) -> Result<Vec<Asset>, String> {
    let inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    Ok(inventory.get_client_assets(&client_id))
}

/// Get demo assets for development
#[tauri::command]
pub async fn get_demo_assets(client_id: String) -> Result<Vec<Asset>, String> {
    Ok(generate_demo_assets(&client_id))
}

/// Get a specific asset by ID
#[tauri::command]
pub async fn get_asset(
    state: State<'_, NetworkState>,
    asset_id: String,
) -> Result<Option<Asset>, String> {
    let inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    Ok(inventory.get_asset(&asset_id).cloned())
}

/// Update asset details request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAssetRequest {
    pub id: String,
    pub name: String,
    pub category: String,
    pub criticality: String,
    pub status: String,
    pub location: Option<String>,
    pub owner: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

/// Update an asset
#[tauri::command]
pub async fn update_asset(
    state: State<'_, NetworkState>,
    request: UpdateAssetRequest,
) -> Result<Asset, String> {
    let mut inventory = state.inventory.lock().map_err(|e| e.to_string())?;

    let existing = inventory.get_asset(&request.id)
        .ok_or_else(|| "Asset not found".to_string())?
        .clone();

    let updated = Asset {
        id: existing.id,
        client_id: existing.client_id,
        name: request.name,
        ip_address: existing.ip_address,
        mac_address: existing.mac_address,
        category: parse_asset_category(&request.category)?,
        operating_system: existing.operating_system,
        criticality: parse_criticality(&request.criticality)?,
        status: parse_asset_status(&request.status)?,
        location: request.location,
        owner: request.owner,
        description: request.description,
        services: existing.services,
        tags: request.tags,
        first_seen: existing.first_seen,
        last_seen: existing.last_seen,
        scan_ids: existing.scan_ids,
        metadata: existing.metadata,
    };

    inventory.update_asset(updated)
}

/// Delete an asset
#[tauri::command]
pub async fn delete_asset(
    state: State<'_, NetworkState>,
    asset_id: String,
) -> Result<bool, String> {
    let mut inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    Ok(inventory.delete_asset(&asset_id))
}

/// Get network statistics for a client
#[tauri::command]
pub async fn get_network_stats(
    state: State<'_, NetworkState>,
    client_id: String,
) -> Result<NetworkStats, String> {
    let inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    Ok(inventory.get_stats(&client_id))
}

// ============================================================================
// Asset Groups Commands
// ============================================================================

/// Create a new asset group
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub client_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn create_asset_group(
    state: State<'_, NetworkState>,
    request: CreateGroupRequest,
) -> Result<AssetGroup, String> {
    let mut inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    Ok(inventory.create_group(&request.client_id, &request.name, request.description))
}

/// Get all groups for a client
#[tauri::command]
pub async fn list_asset_groups(
    state: State<'_, NetworkState>,
    client_id: String,
) -> Result<Vec<AssetGroup>, String> {
    let inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    Ok(inventory.get_client_groups(&client_id))
}

/// Add asset to a group
#[tauri::command]
pub async fn add_asset_to_group(
    state: State<'_, NetworkState>,
    group_id: String,
    asset_id: String,
) -> Result<(), String> {
    let mut inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    inventory.add_to_group(&group_id, &asset_id)
}

/// Remove asset from a group
#[tauri::command]
pub async fn remove_asset_from_group(
    state: State<'_, NetworkState>,
    group_id: String,
    asset_id: String,
) -> Result<(), String> {
    let mut inventory = state.inventory.lock().map_err(|e| e.to_string())?;
    inventory.remove_from_group(&group_id, &asset_id)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_scan_type(s: &str) -> Result<ScanType, String> {
    match s.to_lowercase().as_str() {
        "ping_sweep" | "pingsweep" | "ping" => Ok(ScanType::PingSweep),
        "quick_scan" | "quickscan" | "quick" => Ok(ScanType::QuickScan),
        "standard_scan" | "standardscan" | "standard" => Ok(ScanType::StandardScan),
        "full_scan" | "fullscan" | "full" => Ok(ScanType::FullScan),
        "service_detection" | "servicedetection" | "service" => Ok(ScanType::ServiceDetection),
        "os_detection" | "osdetection" | "os" => Ok(ScanType::OsDetection),
        "vulnerability_scan" | "vulnerabilityscan" | "vuln" => Ok(ScanType::VulnerabilityScan),
        "udp_scan" | "udpscan" | "udp" => Ok(ScanType::UdpScan),
        "custom" => Ok(ScanType::Custom),
        _ => Err(format!("Unknown scan type: {}", s)),
    }
}

fn parse_asset_category(s: &str) -> Result<AssetCategory, String> {
    match s.to_lowercase().as_str() {
        "server" => Ok(AssetCategory::Server),
        "workstation" => Ok(AssetCategory::Workstation),
        "networkdevice" | "network_device" | "network" => Ok(AssetCategory::NetworkDevice),
        "securitydevice" | "security_device" | "security" => Ok(AssetCategory::SecurityDevice),
        "printer" => Ok(AssetCategory::Printer),
        "iot" => Ok(AssetCategory::IoT),
        "mobile" => Ok(AssetCategory::Mobile),
        "virtual" | "vm" => Ok(AssetCategory::Virtual),
        "cloud" => Ok(AssetCategory::Cloud),
        "unknown" => Ok(AssetCategory::Unknown),
        _ => Err(format!("Unknown asset category: {}", s)),
    }
}

fn parse_criticality(s: &str) -> Result<Criticality, String> {
    match s.to_lowercase().as_str() {
        "critical" => Ok(Criticality::Critical),
        "high" => Ok(Criticality::High),
        "medium" => Ok(Criticality::Medium),
        "low" => Ok(Criticality::Low),
        "informational" | "info" => Ok(Criticality::Informational),
        _ => Err(format!("Unknown criticality: {}", s)),
    }
}

fn parse_asset_status(s: &str) -> Result<AssetStatus, String> {
    match s.to_lowercase().as_str() {
        "active" => Ok(AssetStatus::Active),
        "inactive" => Ok(AssetStatus::Inactive),
        "decommissioned" => Ok(AssetStatus::Decommissioned),
        "pending" => Ok(AssetStatus::Pending),
        "maintenance" => Ok(AssetStatus::Maintenance),
        _ => Err(format!("Unknown asset status: {}", s)),
    }
}
