//! Asset Inventory Management
//!
//! Manages discovered network assets, tracks changes over time,
//! and provides asset organization capabilities.

use super::models::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Asset inventory manager for in-memory asset storage
/// In production, this would be backed by the SQLite database
pub struct AssetInventory {
    assets: HashMap<String, Asset>,
    groups: HashMap<String, AssetGroup>,
}

impl AssetInventory {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    /// Add or update an asset from scan results
    pub fn upsert_from_discovery(&mut self, client_id: &str, host: &DiscoveredHost, scan_id: &str) -> Asset {
        let now = chrono::Utc::now().to_rfc3339();

        // Check if asset already exists by IP
        let existing = self.assets.values().find(|a| {
            a.client_id == client_id && a.ip_address == host.ip_address
        });

        if let Some(existing_asset) = existing {
            // Update existing asset
            let mut updated = existing_asset.clone();
            updated.last_seen = now;
            updated.mac_address = host.mac_address.clone().or(updated.mac_address);
            updated.name = host.hostname.clone().unwrap_or(updated.name);

            // Update services from discovered ports
            updated.services = host.ports.iter()
                .filter(|p| matches!(p.state, PortState::Open))
                .map(|p| AssetService {
                    port: p.port,
                    protocol: p.protocol,
                    name: p.service.clone().unwrap_or_else(|| "unknown".to_string()),
                    version: p.product.clone(),
                    state: p.state,
                })
                .collect();

            // Update OS if detected
            if let Some(os) = host.os_matches.first() {
                updated.operating_system = Some(os.name.clone());
            }

            // Add scan ID
            if !updated.scan_ids.contains(&scan_id.to_string()) {
                updated.scan_ids.push(scan_id.to_string());
            }

            self.assets.insert(updated.id.clone(), updated.clone());
            updated
        } else {
            // Create new asset
            let asset = Asset {
                id: Uuid::new_v4().to_string(),
                client_id: client_id.to_string(),
                name: host.hostname.clone().unwrap_or_else(|| host.ip_address.clone()),
                ip_address: host.ip_address.clone(),
                mac_address: host.mac_address.clone(),
                category: infer_category(host),
                operating_system: host.os_matches.first().map(|o| o.name.clone()),
                criticality: Criticality::Medium,
                status: AssetStatus::Active,
                location: None,
                owner: None,
                description: None,
                services: host.ports.iter()
                    .filter(|p| matches!(p.state, PortState::Open))
                    .map(|p| AssetService {
                        port: p.port,
                        protocol: p.protocol,
                        name: p.service.clone().unwrap_or_else(|| "unknown".to_string()),
                        version: p.product.clone(),
                        state: p.state,
                    })
                    .collect(),
                tags: vec![],
                first_seen: now.clone(),
                last_seen: now,
                scan_ids: vec![scan_id.to_string()],
                metadata: None,
            };

            self.assets.insert(asset.id.clone(), asset.clone());
            asset
        }
    }

    /// Get all assets for a client
    pub fn get_client_assets(&self, client_id: &str) -> Vec<Asset> {
        self.assets.values()
            .filter(|a| a.client_id == client_id)
            .cloned()
            .collect()
    }

    /// Get asset by ID
    pub fn get_asset(&self, id: &str) -> Option<&Asset> {
        self.assets.get(id)
    }

    /// Update asset details
    pub fn update_asset(&mut self, asset: Asset) -> Result<Asset, String> {
        if !self.assets.contains_key(&asset.id) {
            return Err("Asset not found".to_string());
        }
        self.assets.insert(asset.id.clone(), asset.clone());
        Ok(asset)
    }

    /// Delete an asset
    pub fn delete_asset(&mut self, id: &str) -> bool {
        self.assets.remove(id).is_some()
    }

    /// Create a new asset group
    pub fn create_group(&mut self, client_id: &str, name: &str, description: Option<String>) -> AssetGroup {
        let group = AssetGroup {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            name: name.to_string(),
            description,
            asset_ids: vec![],
            color: None,
        };
        self.groups.insert(group.id.clone(), group.clone());
        group
    }

    /// Add asset to group
    pub fn add_to_group(&mut self, group_id: &str, asset_id: &str) -> Result<(), String> {
        let group = self.groups.get_mut(group_id)
            .ok_or_else(|| "Group not found".to_string())?;

        if !self.assets.contains_key(asset_id) {
            return Err("Asset not found".to_string());
        }

        if !group.asset_ids.contains(&asset_id.to_string()) {
            group.asset_ids.push(asset_id.to_string());
        }

        Ok(())
    }

    /// Remove asset from group
    pub fn remove_from_group(&mut self, group_id: &str, asset_id: &str) -> Result<(), String> {
        let group = self.groups.get_mut(group_id)
            .ok_or_else(|| "Group not found".to_string())?;

        group.asset_ids.retain(|id| id != asset_id);
        Ok(())
    }

    /// Get groups for a client
    pub fn get_client_groups(&self, client_id: &str) -> Vec<AssetGroup> {
        self.groups.values()
            .filter(|g| g.client_id == client_id)
            .cloned()
            .collect()
    }

    /// Get network statistics for a client
    pub fn get_stats(&self, client_id: &str) -> NetworkStats {
        let assets: Vec<&Asset> = self.assets.values()
            .filter(|a| a.client_id == client_id)
            .collect();

        let total_assets = assets.len();
        let active_assets = assets.iter()
            .filter(|a| matches!(a.status, AssetStatus::Active))
            .count();

        // Count by category
        let mut category_counts: HashMap<AssetCategory, usize> = HashMap::new();
        for asset in &assets {
            *category_counts.entry(asset.category).or_insert(0) += 1;
        }
        let by_category: Vec<CategoryCount> = category_counts.into_iter()
            .map(|(category, count)| CategoryCount { category, count })
            .collect();

        // Count by criticality
        let mut criticality_counts: HashMap<Criticality, usize> = HashMap::new();
        for asset in &assets {
            *criticality_counts.entry(asset.criticality).or_insert(0) += 1;
        }
        let by_criticality: Vec<CriticalityCount> = criticality_counts.into_iter()
            .map(|(criticality, count)| CriticalityCount { criticality, count })
            .collect();

        // Count services
        let mut service_counts: HashMap<(String, u16), usize> = HashMap::new();
        for asset in &assets {
            for service in &asset.services {
                *service_counts.entry((service.name.clone(), service.port)).or_insert(0) += 1;
            }
        }
        let mut top_services: Vec<ServiceCount> = service_counts.into_iter()
            .map(|((service, port), count)| ServiceCount { service, port, count })
            .collect();
        top_services.sort_by(|a, b| b.count.cmp(&a.count));
        top_services.truncate(10);

        NetworkStats {
            total_assets,
            active_assets,
            total_scans: 0, // Would come from scan history
            by_category,
            by_criticality,
            top_services,
            recent_scans: vec![],
        }
    }
}

impl Default for AssetInventory {
    fn default() -> Self {
        Self::new()
    }
}

/// Infer asset category from discovered host data
fn infer_category(host: &DiscoveredHost) -> AssetCategory {
    // Check OS matches first
    if let Some(os) = host.os_matches.first() {
        let os_lower = os.name.to_lowercase();

        // Check device type
        if let Some(ref device_type) = os.device_type {
            let dt_lower = device_type.to_lowercase();
            if dt_lower.contains("router") || dt_lower.contains("switch") {
                return AssetCategory::NetworkDevice;
            }
            if dt_lower.contains("firewall") {
                return AssetCategory::SecurityDevice;
            }
            if dt_lower.contains("printer") {
                return AssetCategory::Printer;
            }
            if dt_lower.contains("phone") || dt_lower.contains("mobile") {
                return AssetCategory::Mobile;
            }
        }

        // Check OS family
        if os_lower.contains("windows server") {
            return AssetCategory::Server;
        }
        if os_lower.contains("windows") {
            return AssetCategory::Workstation;
        }
        if os_lower.contains("linux") || os_lower.contains("ubuntu") || os_lower.contains("centos") {
            // Could be server or workstation, check services
            if has_server_services(&host.ports) {
                return AssetCategory::Server;
            }
        }
        if os_lower.contains("esxi") || os_lower.contains("vmware") {
            return AssetCategory::Virtual;
        }
        if os_lower.contains("ios") || os_lower.contains("android") {
            return AssetCategory::Mobile;
        }
    }

    // Check vendor
    if let Some(ref vendor) = host.vendor {
        let vendor_lower = vendor.to_lowercase();
        if vendor_lower.contains("cisco") || vendor_lower.contains("juniper") ||
           vendor_lower.contains("arista") || vendor_lower.contains("netgear") {
            return AssetCategory::NetworkDevice;
        }
        if vendor_lower.contains("hp") && vendor_lower.contains("printer") {
            return AssetCategory::Printer;
        }
        if vendor_lower.contains("vmware") {
            return AssetCategory::Virtual;
        }
    }

    // Check services to infer type
    if has_server_services(&host.ports) {
        return AssetCategory::Server;
    }

    // Check for network device services
    if has_network_device_services(&host.ports) {
        return AssetCategory::NetworkDevice;
    }

    // Check for printer services
    if has_printer_services(&host.ports) {
        return AssetCategory::Printer;
    }

    AssetCategory::Unknown
}

fn has_server_services(ports: &[DiscoveredPort]) -> bool {
    let server_ports = [22, 80, 443, 3306, 5432, 1433, 1521, 27017, 6379, 8080, 8443];

    ports.iter()
        .filter(|p| matches!(p.state, PortState::Open))
        .any(|p| server_ports.contains(&p.port))
}

fn has_network_device_services(ports: &[DiscoveredPort]) -> bool {
    // Common network device ports
    let network_ports = [23, 161, 162, 179, 830]; // Telnet, SNMP, BGP, NETCONF

    ports.iter()
        .filter(|p| matches!(p.state, PortState::Open))
        .any(|p| network_ports.contains(&p.port))
}

fn has_printer_services(ports: &[DiscoveredPort]) -> bool {
    // Common printer ports
    let printer_ports = [515, 631, 9100]; // LPD, IPP, JetDirect

    ports.iter()
        .filter(|p| matches!(p.state, PortState::Open))
        .any(|p| printer_ports.contains(&p.port))
}

/// Generate demo assets for development/testing
pub fn generate_demo_assets(client_id: &str) -> Vec<Asset> {
    let now = chrono::Utc::now().to_rfc3339();

    vec![
        Asset {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            name: "dc01.corp.local".to_string(),
            ip_address: "192.168.1.10".to_string(),
            mac_address: Some("00:50:56:A1:B2:C3".to_string()),
            category: AssetCategory::Server,
            operating_system: Some("Windows Server 2022".to_string()),
            criticality: Criticality::Critical,
            status: AssetStatus::Active,
            location: Some("Datacenter Rack A1".to_string()),
            owner: Some("IT Infrastructure".to_string()),
            description: Some("Primary Domain Controller".to_string()),
            services: vec![
                AssetService { port: 53, protocol: Protocol::Tcp, name: "DNS".to_string(), version: None, state: PortState::Open },
                AssetService { port: 88, protocol: Protocol::Tcp, name: "Kerberos".to_string(), version: None, state: PortState::Open },
                AssetService { port: 389, protocol: Protocol::Tcp, name: "LDAP".to_string(), version: None, state: PortState::Open },
                AssetService { port: 445, protocol: Protocol::Tcp, name: "SMB".to_string(), version: None, state: PortState::Open },
            ],
            tags: vec!["domain-controller".to_string(), "critical".to_string()],
            first_seen: now.clone(),
            last_seen: now.clone(),
            scan_ids: vec!["demo-scan-1".to_string()],
            metadata: None,
        },
        Asset {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            name: "web-prod-01".to_string(),
            ip_address: "192.168.1.20".to_string(),
            mac_address: Some("00:50:56:A1:D4:E5".to_string()),
            category: AssetCategory::Server,
            operating_system: Some("Ubuntu 22.04 LTS".to_string()),
            criticality: Criticality::High,
            status: AssetStatus::Active,
            location: Some("Datacenter Rack B2".to_string()),
            owner: Some("Web Team".to_string()),
            description: Some("Production Web Server".to_string()),
            services: vec![
                AssetService { port: 22, protocol: Protocol::Tcp, name: "SSH".to_string(), version: Some("OpenSSH 8.9".to_string()), state: PortState::Open },
                AssetService { port: 80, protocol: Protocol::Tcp, name: "HTTP".to_string(), version: Some("nginx 1.24".to_string()), state: PortState::Open },
                AssetService { port: 443, protocol: Protocol::Tcp, name: "HTTPS".to_string(), version: Some("nginx 1.24".to_string()), state: PortState::Open },
            ],
            tags: vec!["web".to_string(), "production".to_string()],
            first_seen: now.clone(),
            last_seen: now.clone(),
            scan_ids: vec!["demo-scan-1".to_string()],
            metadata: None,
        },
        Asset {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            name: "db-prod-01".to_string(),
            ip_address: "192.168.1.30".to_string(),
            mac_address: Some("00:50:56:A1:F6:G7".to_string()),
            category: AssetCategory::Server,
            operating_system: Some("CentOS 8".to_string()),
            criticality: Criticality::Critical,
            status: AssetStatus::Active,
            location: Some("Datacenter Rack C3".to_string()),
            owner: Some("Database Team".to_string()),
            description: Some("Production PostgreSQL Database".to_string()),
            services: vec![
                AssetService { port: 22, protocol: Protocol::Tcp, name: "SSH".to_string(), version: Some("OpenSSH 8.0".to_string()), state: PortState::Open },
                AssetService { port: 5432, protocol: Protocol::Tcp, name: "PostgreSQL".to_string(), version: Some("15.4".to_string()), state: PortState::Open },
            ],
            tags: vec!["database".to_string(), "production".to_string(), "pci".to_string()],
            first_seen: now.clone(),
            last_seen: now.clone(),
            scan_ids: vec!["demo-scan-1".to_string()],
            metadata: None,
        },
        Asset {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            name: "fw-edge-01".to_string(),
            ip_address: "192.168.1.1".to_string(),
            mac_address: Some("00:1A:2B:3C:4D:5E".to_string()),
            category: AssetCategory::SecurityDevice,
            operating_system: Some("Palo Alto PAN-OS 11.0".to_string()),
            criticality: Criticality::Critical,
            status: AssetStatus::Active,
            location: Some("Network Closet A".to_string()),
            owner: Some("Network Security".to_string()),
            description: Some("Edge Firewall".to_string()),
            services: vec![
                AssetService { port: 443, protocol: Protocol::Tcp, name: "HTTPS".to_string(), version: None, state: PortState::Open },
            ],
            tags: vec!["firewall".to_string(), "edge".to_string()],
            first_seen: now.clone(),
            last_seen: now.clone(),
            scan_ids: vec!["demo-scan-1".to_string()],
            metadata: None,
        },
        Asset {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            name: "sw-core-01".to_string(),
            ip_address: "192.168.1.2".to_string(),
            mac_address: Some("00:1B:2C:3D:4E:5F".to_string()),
            category: AssetCategory::NetworkDevice,
            operating_system: Some("Cisco IOS XE 17.6".to_string()),
            criticality: Criticality::High,
            status: AssetStatus::Active,
            location: Some("Network Closet A".to_string()),
            owner: Some("Network Team".to_string()),
            description: Some("Core Switch".to_string()),
            services: vec![
                AssetService { port: 22, protocol: Protocol::Tcp, name: "SSH".to_string(), version: None, state: PortState::Open },
                AssetService { port: 161, protocol: Protocol::Udp, name: "SNMP".to_string(), version: None, state: PortState::Open },
            ],
            tags: vec!["switch".to_string(), "core".to_string()],
            first_seen: now.clone(),
            last_seen: now.clone(),
            scan_ids: vec!["demo-scan-1".to_string()],
            metadata: None,
        },
        Asset {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            name: "WS-JSMITH".to_string(),
            ip_address: "192.168.1.100".to_string(),
            mac_address: Some("00:50:56:B1:C2:D3".to_string()),
            category: AssetCategory::Workstation,
            operating_system: Some("Windows 11 Pro".to_string()),
            criticality: Criticality::Medium,
            status: AssetStatus::Active,
            location: Some("Floor 2, Desk 215".to_string()),
            owner: Some("John Smith".to_string()),
            description: Some("Developer Workstation".to_string()),
            services: vec![
                AssetService { port: 135, protocol: Protocol::Tcp, name: "MSRPC".to_string(), version: None, state: PortState::Open },
                AssetService { port: 445, protocol: Protocol::Tcp, name: "SMB".to_string(), version: None, state: PortState::Open },
            ],
            tags: vec!["workstation".to_string(), "developer".to_string()],
            first_seen: now.clone(),
            last_seen: now.clone(),
            scan_ids: vec!["demo-scan-1".to_string()],
            metadata: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_inventory_add() {
        let mut inventory = AssetInventory::new();

        let host = DiscoveredHost {
            ip_address: "192.168.1.100".to_string(),
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            hostname: Some("test-server".to_string()),
            vendor: Some("VMware".to_string()),
            status: "up".to_string(),
            ports: vec![
                DiscoveredPort {
                    port: 22,
                    protocol: Protocol::Tcp,
                    state: PortState::Open,
                    service: Some("ssh".to_string()),
                    product: Some("OpenSSH".to_string()),
                    version: Some("8.9".to_string()),
                    extra_info: None,
                    scripts: vec![],
                },
            ],
            os_matches: vec![],
            host_scripts: vec![],
        };

        let asset = inventory.upsert_from_discovery("client-1", &host, "scan-1");

        assert_eq!(asset.name, "test-server");
        assert_eq!(asset.ip_address, "192.168.1.100");
        assert_eq!(asset.services.len(), 1);
    }

    #[test]
    fn test_infer_category_server() {
        let host = DiscoveredHost {
            ip_address: "192.168.1.10".to_string(),
            mac_address: None,
            hostname: None,
            vendor: None,
            status: "up".to_string(),
            ports: vec![
                DiscoveredPort {
                    port: 443,
                    protocol: Protocol::Tcp,
                    state: PortState::Open,
                    service: Some("https".to_string()),
                    product: None,
                    version: None,
                    extra_info: None,
                    scripts: vec![],
                },
            ],
            os_matches: vec![],
            host_scripts: vec![],
        };

        assert_eq!(infer_category(&host), AssetCategory::Server);
    }
}
