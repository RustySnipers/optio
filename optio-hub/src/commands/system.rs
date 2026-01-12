//! System Commands
//!
//! System information and utility commands.

use serde::Serialize;
use std::net::UdpSocket;

/// System information response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
    pub username: String,
    pub app_version: String,
    pub local_ip: Option<String>,
}

/// Get system information
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let os_name = std::env::consts::OS.to_string();
    let os_version = get_os_version();
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let username = whoami::username();
    let local_ip = detect_local_ip();

    Ok(SystemInfo {
        os_name,
        os_version,
        hostname,
        username,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        local_ip,
    })
}

/// Get the consultant's local IP address for script injection
#[tauri::command]
pub async fn get_consultant_ip() -> Result<String, String> {
    detect_local_ip().ok_or_else(|| "Could not detect local IP address".to_string())
}

/// Detect the local IP address by creating a UDP socket
/// This doesn't actually send data, just uses the OS routing table
pub fn detect_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    // Connect to a public IP to determine the local interface
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

fn get_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        // Try to get Windows version info
        "Windows".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| format!("macOS {}", s.trim()))
            .unwrap_or_else(|| "macOS".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .map(|line| {
                        line.trim_start_matches("PRETTY_NAME=")
                            .trim_matches('"')
                            .to_string()
                    })
            })
            .unwrap_or_else(|| "Linux".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Unknown".to_string()
    }
}
