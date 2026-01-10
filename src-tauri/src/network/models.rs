//! Network Intelligence Data Models
//!
//! Types for network scanning, asset discovery, and inventory management.

use serde::{Deserialize, Serialize};

// ============================================================================
// Scan Types
// ============================================================================

/// Type of network scan to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    /// Quick ping sweep to discover live hosts
    PingSweep,
    /// Basic port scan (top 100 ports)
    QuickScan,
    /// Standard port scan (top 1000 ports)
    StandardScan,
    /// Full TCP port scan (all 65535 ports)
    FullScan,
    /// Service version detection
    ServiceDetection,
    /// OS detection scan
    OsDetection,
    /// Vulnerability scan using NSE scripts
    VulnerabilityScan,
    /// UDP port scan
    UdpScan,
    /// Custom scan with user-defined options
    Custom,
}

impl ScanType {
    /// Get the Nmap arguments for this scan type
    pub fn to_nmap_args(&self) -> Vec<&'static str> {
        match self {
            ScanType::PingSweep => vec!["-sn", "-PE", "-PP", "-PM"],
            ScanType::QuickScan => vec!["-sS", "-T4", "--top-ports", "100"],
            ScanType::StandardScan => vec!["-sS", "-sV", "-T4"],
            ScanType::FullScan => vec!["-sS", "-p-", "-T4"],
            ScanType::ServiceDetection => vec!["-sS", "-sV", "-sC", "-T4"],
            ScanType::OsDetection => vec!["-sS", "-O", "-T4"],
            ScanType::VulnerabilityScan => vec!["-sS", "-sV", "--script=vuln", "-T4"],
            ScanType::UdpScan => vec!["-sU", "--top-ports", "100", "-T4"],
            ScanType::Custom => vec![],
        }
    }

    /// Get display name for the scan type
    pub fn display_name(&self) -> &'static str {
        match self {
            ScanType::PingSweep => "Ping Sweep",
            ScanType::QuickScan => "Quick Scan",
            ScanType::StandardScan => "Standard Scan",
            ScanType::FullScan => "Full Port Scan",
            ScanType::ServiceDetection => "Service Detection",
            ScanType::OsDetection => "OS Detection",
            ScanType::VulnerabilityScan => "Vulnerability Scan",
            ScanType::UdpScan => "UDP Scan",
            ScanType::Custom => "Custom Scan",
        }
    }

    /// Get description of what this scan does
    pub fn description(&self) -> &'static str {
        match self {
            ScanType::PingSweep => "Fast host discovery using ICMP echo, timestamp, and netmask requests",
            ScanType::QuickScan => "TCP SYN scan of top 100 most common ports",
            ScanType::StandardScan => "TCP SYN scan with service version detection on top 1000 ports",
            ScanType::FullScan => "Complete TCP scan of all 65,535 ports",
            ScanType::ServiceDetection => "Service version detection with default NSE scripts",
            ScanType::OsDetection => "Operating system fingerprinting using TCP/IP stack analysis",
            ScanType::VulnerabilityScan => "Vulnerability detection using NSE vuln scripts",
            ScanType::UdpScan => "UDP scan of top 100 common UDP ports",
            ScanType::Custom => "Custom scan with user-defined Nmap options",
        }
    }

    /// Estimated duration category
    pub fn duration_estimate(&self) -> &'static str {
        match self {
            ScanType::PingSweep => "Fast (1-5 min)",
            ScanType::QuickScan => "Fast (2-10 min)",
            ScanType::StandardScan => "Medium (5-30 min)",
            ScanType::FullScan => "Slow (30-120 min)",
            ScanType::ServiceDetection => "Medium (10-45 min)",
            ScanType::OsDetection => "Medium (5-20 min)",
            ScanType::VulnerabilityScan => "Slow (30-90 min)",
            ScanType::UdpScan => "Slow (15-60 min)",
            ScanType::Custom => "Variable",
        }
    }
}

/// Status of a scan job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    /// Scan is queued but not yet started
    Queued,
    /// Scan is currently running
    Running,
    /// Scan completed successfully
    Completed,
    /// Scan failed with an error
    Failed,
    /// Scan was cancelled by user
    Cancelled,
}

/// Port state as reported by Nmap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortState {
    Open,
    Closed,
    Filtered,
    Unfiltered,
    OpenFiltered,
    ClosedFiltered,
}

/// Protocol type for port
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
    Sctp,
}

// ============================================================================
// Scan Configuration
// ============================================================================

/// Configuration for a network scan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    /// Target specification (IP, range, CIDR, hostname)
    pub targets: Vec<String>,
    /// Type of scan to perform
    pub scan_type: ScanType,
    /// Custom Nmap arguments (for custom scan type)
    pub custom_args: Option<String>,
    /// Specific ports to scan (overrides scan type defaults)
    pub ports: Option<String>,
    /// Exclude these targets from scanning
    pub exclude_targets: Option<Vec<String>>,
    /// Enable aggressive timing (faster but noisier)
    pub aggressive: bool,
    /// Skip host discovery (treat all hosts as online)
    pub skip_discovery: bool,
    /// Output format preferences
    pub output_formats: Vec<OutputFormat>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            targets: vec![],
            scan_type: ScanType::QuickScan,
            custom_args: None,
            ports: None,
            exclude_targets: None,
            aggressive: false,
            skip_discovery: false,
            output_formats: vec![OutputFormat::Normal, OutputFormat::Xml],
        }
    }
}

/// Output format for scan results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Normal,
    Xml,
    Grepable,
    Json,
}

// ============================================================================
// Scan Results
// ============================================================================

/// A network scan job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanJob {
    /// Unique identifier for this scan
    pub id: String,
    /// Client ID this scan belongs to
    pub client_id: String,
    /// Human-readable name for the scan
    pub name: String,
    /// Scan configuration
    pub config: ScanConfig,
    /// Current status
    pub status: ScanStatus,
    /// When the scan was created
    pub created_at: String,
    /// When the scan started running
    pub started_at: Option<String>,
    /// When the scan completed
    pub completed_at: Option<String>,
    /// Error message if scan failed
    pub error: Option<String>,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Raw Nmap output
    pub raw_output: Option<String>,
}

/// Results from a completed scan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResults {
    /// Scan job ID
    pub scan_id: String,
    /// Discovered hosts
    pub hosts: Vec<DiscoveredHost>,
    /// Total hosts scanned
    pub hosts_scanned: u32,
    /// Hosts that were up/responsive
    pub hosts_up: u32,
    /// Scan duration in seconds
    pub duration_seconds: f64,
    /// Nmap version used
    pub nmap_version: Option<String>,
    /// Command line that was executed
    pub command_line: String,
    /// Scan start time
    pub start_time: String,
    /// Scan end time
    pub end_time: String,
}

/// A discovered host from a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredHost {
    /// IP address (v4 or v6)
    pub ip_address: String,
    /// MAC address if available
    pub mac_address: Option<String>,
    /// Hostname if resolved
    pub hostname: Option<String>,
    /// Vendor from MAC address lookup
    pub vendor: Option<String>,
    /// Host status (up/down)
    pub status: String,
    /// Discovered open ports
    pub ports: Vec<DiscoveredPort>,
    /// OS detection results
    pub os_matches: Vec<OsMatch>,
    /// Additional host scripts output
    pub host_scripts: Vec<ScriptResult>,
}

/// A discovered port on a host
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredPort {
    /// Port number
    pub port: u16,
    /// Protocol (TCP/UDP)
    pub protocol: Protocol,
    /// Port state
    pub state: PortState,
    /// Service name
    pub service: Option<String>,
    /// Service product/version
    pub product: Option<String>,
    /// Product version
    pub version: Option<String>,
    /// Extra service info
    pub extra_info: Option<String>,
    /// NSE script results for this port
    pub scripts: Vec<ScriptResult>,
}

/// OS match from detection scan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsMatch {
    /// OS name
    pub name: String,
    /// Accuracy percentage
    pub accuracy: u8,
    /// OS family (Windows, Linux, etc.)
    pub os_family: Option<String>,
    /// OS generation
    pub os_gen: Option<String>,
    /// Device type
    pub device_type: Option<String>,
}

/// NSE script result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptResult {
    /// Script name
    pub id: String,
    /// Script output
    pub output: String,
}

// ============================================================================
// Asset Inventory
// ============================================================================

/// Asset category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    Server,
    Workstation,
    NetworkDevice,
    SecurityDevice,
    Printer,
    IoT,
    Mobile,
    Virtual,
    Cloud,
    Unknown,
}

impl AssetCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            AssetCategory::Server => "Server",
            AssetCategory::Workstation => "Workstation",
            AssetCategory::NetworkDevice => "Network Device",
            AssetCategory::SecurityDevice => "Security Device",
            AssetCategory::Printer => "Printer",
            AssetCategory::IoT => "IoT Device",
            AssetCategory::Mobile => "Mobile Device",
            AssetCategory::Virtual => "Virtual Machine",
            AssetCategory::Cloud => "Cloud Resource",
            AssetCategory::Unknown => "Unknown",
        }
    }
}

/// Asset criticality level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criticality {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

/// Asset status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Active,
    Inactive,
    Decommissioned,
    Pending,
    Maintenance,
}

/// An asset in the inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Unique asset identifier
    pub id: String,
    /// Client this asset belongs to
    pub client_id: String,
    /// Asset name/hostname
    pub name: String,
    /// IP address
    pub ip_address: String,
    /// MAC address
    pub mac_address: Option<String>,
    /// Asset category
    pub category: AssetCategory,
    /// Operating system
    pub operating_system: Option<String>,
    /// Criticality level
    pub criticality: Criticality,
    /// Current status
    pub status: AssetStatus,
    /// Physical/logical location
    pub location: Option<String>,
    /// Business owner
    pub owner: Option<String>,
    /// Description/notes
    pub description: Option<String>,
    /// Associated services
    pub services: Vec<AssetService>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// When first discovered
    pub first_seen: String,
    /// When last seen in a scan
    pub last_seen: String,
    /// Scan IDs where this asset was found
    pub scan_ids: Vec<String>,
    /// Custom metadata
    pub metadata: Option<serde_json::Value>,
}

/// A service running on an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetService {
    /// Port number
    pub port: u16,
    /// Protocol
    pub protocol: Protocol,
    /// Service name
    pub name: String,
    /// Product/version info
    pub version: Option<String>,
    /// Service state
    pub state: PortState,
}

/// Asset group for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetGroup {
    /// Unique group identifier
    pub id: String,
    /// Client this group belongs to
    pub client_id: String,
    /// Group name
    pub name: String,
    /// Group description
    pub description: Option<String>,
    /// Asset IDs in this group
    pub asset_ids: Vec<String>,
    /// Color for UI display
    pub color: Option<String>,
}

// ============================================================================
// Statistics
// ============================================================================

/// Network discovery statistics for a client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStats {
    /// Total assets discovered
    pub total_assets: usize,
    /// Active assets
    pub active_assets: usize,
    /// Total scans performed
    pub total_scans: usize,
    /// Assets by category
    pub by_category: Vec<CategoryCount>,
    /// Assets by criticality
    pub by_criticality: Vec<CriticalityCount>,
    /// Top services discovered
    pub top_services: Vec<ServiceCount>,
    /// Recent scan activity
    pub recent_scans: Vec<ScanSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCount {
    pub category: AssetCategory,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriticalityCount {
    pub criticality: Criticality,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCount {
    pub service: String,
    pub port: u16,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub id: String,
    pub name: String,
    pub scan_type: ScanType,
    pub status: ScanStatus,
    pub hosts_found: u32,
    pub completed_at: Option<String>,
}
