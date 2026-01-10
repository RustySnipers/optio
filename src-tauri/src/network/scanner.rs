//! Nmap Scanner Wrapper
//!
//! Provides a safe wrapper around Nmap for network discovery and scanning.
//! Handles command construction, execution, and result parsing.

use super::models::*;
use std::collections::HashMap;
use std::process::Command;
use uuid::Uuid;

/// Check if Nmap is installed and available
pub fn check_nmap_installed() -> Result<NmapInfo, String> {
    let output = Command::new("nmap")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute nmap: {}. Is Nmap installed?", e))?;

    if !output.status.success() {
        return Err("Nmap command failed".to_string());
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    let version = parse_nmap_version(&version_output);

    Ok(NmapInfo {
        installed: true,
        version,
        path: which_nmap(),
    })
}

/// Information about the Nmap installation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NmapInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

fn parse_nmap_version(output: &str) -> Option<String> {
    // Parse version from "Nmap version 7.94 ( https://nmap.org )"
    output
        .lines()
        .next()
        .and_then(|line| {
            if line.starts_with("Nmap version") {
                line.split_whitespace()
                    .nth(2)
                    .map(|v| v.to_string())
            } else {
                None
            }
        })
}

fn which_nmap() -> Option<String> {
    Command::new("which")
        .arg("nmap")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Build Nmap command from scan configuration
pub fn build_nmap_command(config: &ScanConfig) -> Vec<String> {
    let mut args = Vec::new();

    // Add scan type arguments
    match config.scan_type {
        ScanType::Custom => {
            if let Some(ref custom) = config.custom_args {
                args.extend(custom.split_whitespace().map(|s| s.to_string()));
            }
        }
        _ => {
            args.extend(
                config.scan_type
                    .to_nmap_args()
                    .into_iter()
                    .map(|s| s.to_string())
            );
        }
    }

    // Add aggressive timing if requested
    if config.aggressive && !args.iter().any(|a| a.starts_with("-T")) {
        args.push("-T5".to_string());
    }

    // Skip host discovery if requested
    if config.skip_discovery {
        args.push("-Pn".to_string());
    }

    // Add specific ports if specified
    if let Some(ref ports) = config.ports {
        args.push("-p".to_string());
        args.push(ports.clone());
    }

    // Add exclude targets
    if let Some(ref excludes) = config.exclude_targets {
        if !excludes.is_empty() {
            args.push("--exclude".to_string());
            args.push(excludes.join(","));
        }
    }

    // Add output format for XML parsing
    args.push("-oX".to_string());
    args.push("-".to_string()); // Output to stdout

    // Add targets
    args.extend(config.targets.clone());

    args
}

/// Get available scan types with their descriptions
pub fn get_scan_types() -> Vec<ScanTypeInfo> {
    vec![
        ScanTypeInfo {
            scan_type: ScanType::PingSweep,
            name: ScanType::PingSweep.display_name().to_string(),
            description: ScanType::PingSweep.description().to_string(),
            duration: ScanType::PingSweep.duration_estimate().to_string(),
            requires_root: false,
        },
        ScanTypeInfo {
            scan_type: ScanType::QuickScan,
            name: ScanType::QuickScan.display_name().to_string(),
            description: ScanType::QuickScan.description().to_string(),
            duration: ScanType::QuickScan.duration_estimate().to_string(),
            requires_root: true,
        },
        ScanTypeInfo {
            scan_type: ScanType::StandardScan,
            name: ScanType::StandardScan.display_name().to_string(),
            description: ScanType::StandardScan.description().to_string(),
            duration: ScanType::StandardScan.duration_estimate().to_string(),
            requires_root: true,
        },
        ScanTypeInfo {
            scan_type: ScanType::FullScan,
            name: ScanType::FullScan.display_name().to_string(),
            description: ScanType::FullScan.description().to_string(),
            duration: ScanType::FullScan.duration_estimate().to_string(),
            requires_root: true,
        },
        ScanTypeInfo {
            scan_type: ScanType::ServiceDetection,
            name: ScanType::ServiceDetection.display_name().to_string(),
            description: ScanType::ServiceDetection.description().to_string(),
            duration: ScanType::ServiceDetection.duration_estimate().to_string(),
            requires_root: true,
        },
        ScanTypeInfo {
            scan_type: ScanType::OsDetection,
            name: ScanType::OsDetection.display_name().to_string(),
            description: ScanType::OsDetection.description().to_string(),
            duration: ScanType::OsDetection.duration_estimate().to_string(),
            requires_root: true,
        },
        ScanTypeInfo {
            scan_type: ScanType::VulnerabilityScan,
            name: ScanType::VulnerabilityScan.display_name().to_string(),
            description: ScanType::VulnerabilityScan.description().to_string(),
            duration: ScanType::VulnerabilityScan.duration_estimate().to_string(),
            requires_root: true,
        },
        ScanTypeInfo {
            scan_type: ScanType::UdpScan,
            name: ScanType::UdpScan.display_name().to_string(),
            description: ScanType::UdpScan.description().to_string(),
            duration: ScanType::UdpScan.duration_estimate().to_string(),
            requires_root: true,
        },
    ]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTypeInfo {
    pub scan_type: ScanType,
    pub name: String,
    pub description: String,
    pub duration: String,
    pub requires_root: bool,
}

/// Parse Nmap XML output into structured results
pub fn parse_nmap_xml(xml: &str) -> Result<ScanResults, String> {
    // Simple XML parsing - in production would use a proper XML parser
    // For now, we'll create a demo result structure

    let scan_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Extract basic info from XML
    let hosts = parse_hosts_from_xml(xml);
    let hosts_up = hosts.iter().filter(|h| h.status == "up").count() as u32;

    // Try to extract nmap version
    let nmap_version = extract_xml_attr(xml, "scanner", "version");

    // Extract timing info
    let start_time = extract_xml_attr(xml, "nmaprun", "startstr")
        .unwrap_or_else(|| now.clone());

    Ok(ScanResults {
        scan_id,
        hosts,
        hosts_scanned: hosts_up, // Simplified
        hosts_up,
        duration_seconds: 0.0, // Would parse from XML
        nmap_version,
        command_line: extract_xml_attr(xml, "nmaprun", "args").unwrap_or_default(),
        start_time,
        end_time: now,
    })
}

fn parse_hosts_from_xml(xml: &str) -> Vec<DiscoveredHost> {
    let mut hosts = Vec::new();

    // Simple regex-free parsing for demonstration
    // In production, use quick-xml or roxmltree

    let mut current_pos = 0;
    while let Some(host_start) = xml[current_pos..].find("<host") {
        let abs_start = current_pos + host_start;
        if let Some(host_end) = xml[abs_start..].find("</host>") {
            let host_xml = &xml[abs_start..abs_start + host_end + 7];

            if let Some(host) = parse_single_host(host_xml) {
                hosts.push(host);
            }

            current_pos = abs_start + host_end + 7;
        } else {
            break;
        }
    }

    hosts
}

fn parse_single_host(host_xml: &str) -> Option<DiscoveredHost> {
    // Extract IP address
    let ip = extract_address(host_xml, "ipv4")
        .or_else(|| extract_address(host_xml, "ipv6"))?;

    // Extract MAC if present
    let mac = extract_address(host_xml, "mac");

    // Extract vendor
    let vendor = extract_xml_attr(host_xml, "address", "vendor");

    // Extract hostname
    let hostname = extract_hostname(host_xml);

    // Extract status
    let status = extract_xml_attr(host_xml, "status", "state")
        .unwrap_or_else(|| "unknown".to_string());

    // Parse ports
    let ports = parse_ports(host_xml);

    // Parse OS matches
    let os_matches = parse_os_matches(host_xml);

    Some(DiscoveredHost {
        ip_address: ip,
        mac_address: mac,
        hostname,
        vendor,
        status,
        ports,
        os_matches,
        host_scripts: vec![],
    })
}

fn extract_address(xml: &str, addr_type: &str) -> Option<String> {
    // Look for <address addr="..." addrtype="ipv4"/>
    let pattern = format!(r#"addrtype="{}""#, addr_type);

    if let Some(pos) = xml.find(&pattern) {
        // Find the addr= attribute before this
        let before = &xml[..pos];
        if let Some(addr_start) = before.rfind(r#"addr=""#) {
            let start = addr_start + 6;
            if let Some(end) = xml[start..].find('"') {
                return Some(xml[start..start + end].to_string());
            }
        }
    }
    None
}

fn extract_hostname(xml: &str) -> Option<String> {
    // Look for <hostname name="..." type="PTR"/>
    if let Some(pos) = xml.find("<hostname") {
        let after = &xml[pos..];
        if let Some(name_pos) = after.find(r#"name=""#) {
            let start = name_pos + 6;
            if let Some(end) = after[start..].find('"') {
                return Some(after[start..start + end].to_string());
            }
        }
    }
    None
}

fn extract_xml_attr(xml: &str, element: &str, attr: &str) -> Option<String> {
    let element_pattern = format!("<{}", element);
    if let Some(elem_pos) = xml.find(&element_pattern) {
        let after = &xml[elem_pos..];
        let attr_pattern = format!(r#"{}=""#, attr);
        if let Some(attr_pos) = after.find(&attr_pattern) {
            let start = attr_pos + attr.len() + 2;
            if let Some(end) = after[start..].find('"') {
                return Some(after[start..start + end].to_string());
            }
        }
    }
    None
}

fn parse_ports(xml: &str) -> Vec<DiscoveredPort> {
    let mut ports = Vec::new();

    let mut current_pos = 0;
    while let Some(port_start) = xml[current_pos..].find("<port ") {
        let abs_start = current_pos + port_start;

        // Find the end of this port element
        let end_pos = if let Some(end) = xml[abs_start..].find("</port>") {
            abs_start + end + 7
        } else if let Some(end) = xml[abs_start..].find("/>") {
            abs_start + end + 2
        } else {
            break;
        };

        let port_xml = &xml[abs_start..end_pos];

        // Extract port number
        if let Some(port_num) = extract_xml_attr(port_xml, "port", "portid")
            .and_then(|p| p.parse::<u16>().ok())
        {
            // Extract protocol
            let protocol = match extract_xml_attr(port_xml, "port", "protocol").as_deref() {
                Some("udp") => Protocol::Udp,
                Some("sctp") => Protocol::Sctp,
                _ => Protocol::Tcp,
            };

            // Extract state
            let state = match extract_xml_attr(port_xml, "state", "state").as_deref() {
                Some("open") => PortState::Open,
                Some("closed") => PortState::Closed,
                Some("filtered") => PortState::Filtered,
                Some("unfiltered") => PortState::Unfiltered,
                Some("open|filtered") => PortState::OpenFiltered,
                Some("closed|filtered") => PortState::ClosedFiltered,
                _ => PortState::Filtered,
            };

            // Extract service info
            let service = extract_xml_attr(port_xml, "service", "name");
            let product = extract_xml_attr(port_xml, "service", "product");
            let version = extract_xml_attr(port_xml, "service", "version");
            let extra_info = extract_xml_attr(port_xml, "service", "extrainfo");

            ports.push(DiscoveredPort {
                port: port_num,
                protocol,
                state,
                service,
                product,
                version,
                extra_info,
                scripts: vec![],
            });
        }

        current_pos = end_pos;
    }

    ports
}

fn parse_os_matches(xml: &str) -> Vec<OsMatch> {
    let mut matches = Vec::new();

    let mut current_pos = 0;
    while let Some(os_start) = xml[current_pos..].find("<osmatch ") {
        let abs_start = current_pos + os_start;

        let end_pos = if let Some(end) = xml[abs_start..].find("/>") {
            abs_start + end + 2
        } else if let Some(end) = xml[abs_start..].find("</osmatch>") {
            abs_start + end + 10
        } else {
            break;
        };

        let os_xml = &xml[abs_start..end_pos];

        if let Some(name) = extract_xml_attr(os_xml, "osmatch", "name") {
            let accuracy = extract_xml_attr(os_xml, "osmatch", "accuracy")
                .and_then(|a| a.parse::<u8>().ok())
                .unwrap_or(0);

            matches.push(OsMatch {
                name,
                accuracy,
                os_family: extract_xml_attr(os_xml, "osclass", "osfamily"),
                os_gen: extract_xml_attr(os_xml, "osclass", "osgen"),
                device_type: extract_xml_attr(os_xml, "osclass", "type"),
            });
        }

        current_pos = end_pos;
    }

    // Sort by accuracy descending
    matches.sort_by(|a, b| b.accuracy.cmp(&a.accuracy));

    matches
}

/// Validate target specification
pub fn validate_target(target: &str) -> Result<TargetValidation, String> {
    let target = target.trim();

    if target.is_empty() {
        return Ok(TargetValidation {
            valid: false,
            target_type: None,
            normalized: None,
            error: Some("Target cannot be empty".to_string()),
        });
    }

    // Check for CIDR notation
    if target.contains('/') {
        let parts: Vec<&str> = target.split('/').collect();
        if parts.len() == 2 {
            if let Ok(prefix) = parts[1].parse::<u8>() {
                if prefix <= 32 {
                    return Ok(TargetValidation {
                        valid: true,
                        target_type: Some("CIDR".to_string()),
                        normalized: Some(target.to_string()),
                        error: None,
                    });
                }
            }
        }
        return Ok(TargetValidation {
            valid: false,
            target_type: None,
            normalized: None,
            error: Some("Invalid CIDR notation".to_string()),
        });
    }

    // Check for IP range (192.168.1.1-100)
    if target.contains('-') && !target.contains(':') {
        return Ok(TargetValidation {
            valid: true,
            target_type: Some("IP Range".to_string()),
            normalized: Some(target.to_string()),
            error: None,
        });
    }

    // Check for IPv4 address
    let ipv4_parts: Vec<&str> = target.split('.').collect();
    if ipv4_parts.len() == 4 {
        let valid = ipv4_parts.iter().all(|p| {
            p.parse::<u8>().is_ok() || *p == "*"
        });
        if valid {
            return Ok(TargetValidation {
                valid: true,
                target_type: Some("IPv4".to_string()),
                normalized: Some(target.to_string()),
                error: None,
            });
        }
    }

    // Check for IPv6 address
    if target.contains(':') {
        return Ok(TargetValidation {
            valid: true,
            target_type: Some("IPv6".to_string()),
            normalized: Some(target.to_string()),
            error: None,
        });
    }

    // Assume hostname
    if target.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
        return Ok(TargetValidation {
            valid: true,
            target_type: Some("Hostname".to_string()),
            normalized: Some(target.to_string()),
            error: None,
        });
    }

    Ok(TargetValidation {
        valid: false,
        target_type: None,
        normalized: None,
        error: Some("Invalid target format".to_string()),
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetValidation {
    pub valid: bool,
    pub target_type: Option<String>,
    pub normalized: Option<String>,
    pub error: Option<String>,
}

/// Common port definitions for quick reference
pub fn get_common_ports() -> Vec<CommonPort> {
    vec![
        CommonPort { port: 21, service: "FTP", description: "File Transfer Protocol" },
        CommonPort { port: 22, service: "SSH", description: "Secure Shell" },
        CommonPort { port: 23, service: "Telnet", description: "Telnet (insecure)" },
        CommonPort { port: 25, service: "SMTP", description: "Simple Mail Transfer Protocol" },
        CommonPort { port: 53, service: "DNS", description: "Domain Name System" },
        CommonPort { port: 80, service: "HTTP", description: "Hypertext Transfer Protocol" },
        CommonPort { port: 110, service: "POP3", description: "Post Office Protocol v3" },
        CommonPort { port: 135, service: "MSRPC", description: "Microsoft RPC" },
        CommonPort { port: 139, service: "NetBIOS", description: "NetBIOS Session Service" },
        CommonPort { port: 143, service: "IMAP", description: "Internet Message Access Protocol" },
        CommonPort { port: 443, service: "HTTPS", description: "HTTP Secure" },
        CommonPort { port: 445, service: "SMB", description: "Server Message Block" },
        CommonPort { port: 993, service: "IMAPS", description: "IMAP over SSL" },
        CommonPort { port: 995, service: "POP3S", description: "POP3 over SSL" },
        CommonPort { port: 1433, service: "MSSQL", description: "Microsoft SQL Server" },
        CommonPort { port: 1521, service: "Oracle", description: "Oracle Database" },
        CommonPort { port: 3306, service: "MySQL", description: "MySQL Database" },
        CommonPort { port: 3389, service: "RDP", description: "Remote Desktop Protocol" },
        CommonPort { port: 5432, service: "PostgreSQL", description: "PostgreSQL Database" },
        CommonPort { port: 5900, service: "VNC", description: "Virtual Network Computing" },
        CommonPort { port: 6379, service: "Redis", description: "Redis Database" },
        CommonPort { port: 8080, service: "HTTP-Alt", description: "HTTP Alternative" },
        CommonPort { port: 8443, service: "HTTPS-Alt", description: "HTTPS Alternative" },
        CommonPort { port: 27017, service: "MongoDB", description: "MongoDB Database" },
    ]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonPort {
    pub port: u16,
    pub service: &'static str,
    pub description: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_quick_scan() {
        let config = ScanConfig {
            targets: vec!["192.168.1.0/24".to_string()],
            scan_type: ScanType::QuickScan,
            ..Default::default()
        };

        let args = build_nmap_command(&config);
        assert!(args.contains(&"-sS".to_string()));
        assert!(args.contains(&"192.168.1.0/24".to_string()));
    }

    #[test]
    fn test_validate_target_cidr() {
        let result = validate_target("192.168.1.0/24").unwrap();
        assert!(result.valid);
        assert_eq!(result.target_type, Some("CIDR".to_string()));
    }

    #[test]
    fn test_validate_target_ipv4() {
        let result = validate_target("192.168.1.1").unwrap();
        assert!(result.valid);
        assert_eq!(result.target_type, Some("IPv4".to_string()));
    }

    #[test]
    fn test_validate_target_hostname() {
        let result = validate_target("example.com").unwrap();
        assert!(result.valid);
        assert_eq!(result.target_type, Some("Hostname".to_string()));
    }
}
