//! Optio - Consultant-in-a-Box Backend
//!
//! High-performance, local-first security toolkit for Enterprise Architects
//! and IT Security Consultants.

pub mod commands;
pub mod factory;
pub mod grc;
pub mod infrastructure;
pub mod network;
pub mod error;
pub mod db;

use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the Tauri application with all plugins and commands
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for structured logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "optio=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Optio v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(commands::network::NetworkState::default())
        .setup(|app| {
            // Initialize the database on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::initialize(&app_handle).await {
                    tracing::error!("Failed to initialize database: {}", e);
                }
            });

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Factory commands
            commands::factory::generate_client_script,
            commands::factory::list_templates,
            commands::factory::get_script_preview,
            commands::factory::validate_config,
            // Client management commands
            commands::clients::create_client,
            commands::clients::list_clients,
            commands::clients::get_client,
            commands::clients::update_client,
            commands::clients::delete_client,
            // System commands
            commands::system::get_system_info,
            commands::system::get_consultant_ip,
            // GRC commands
            commands::grc::list_frameworks,
            commands::grc::get_framework_controls_cmd,
            commands::grc::create_assessment,
            commands::grc::get_assessment,
            commands::grc::list_client_assessments,
            commands::grc::list_assessments,
            commands::grc::update_assessment_status,
            commands::grc::delete_assessment,
            commands::grc::update_control_assessment,
            commands::grc::get_control_assessments,
            commands::grc::batch_update_controls,
            commands::grc::create_evidence,
            commands::grc::get_assessment_evidence,
            commands::grc::delete_evidence,
            commands::grc::get_assessment_summary,
            // Infrastructure commands
            commands::infrastructure::get_cloud_readiness_items,
            commands::infrastructure::get_cloud_readiness_by_category,
            commands::infrastructure::assess_cloud_readiness,
            commands::infrastructure::get_k8s_hardening_checklist,
            commands::infrastructure::get_k8s_hardening_by_category,
            commands::infrastructure::audit_k8s_hardening,
            commands::infrastructure::get_k8s_severity_stats,
            commands::infrastructure::get_finops_templates,
            commands::infrastructure::calculate_single_resource_cost,
            commands::infrastructure::generate_finops_report,
            commands::infrastructure::compare_cloud_providers,
            // Network Intelligence commands
            commands::network::check_nmap,
            commands::network::get_scan_type_list,
            commands::network::get_common_port_list,
            commands::network::validate_scan_target,
            commands::network::create_scan,
            commands::network::preview_scan_command,
            commands::network::list_scans,
            commands::network::get_scan,
            commands::network::delete_scan,
            commands::network::list_assets,
            commands::network::get_demo_assets,
            commands::network::get_asset,
            commands::network::update_asset,
            commands::network::delete_asset,
            commands::network::get_network_stats,
            commands::network::create_asset_group,
            commands::network::list_asset_groups,
            commands::network::add_asset_to_group,
            commands::network::remove_asset_from_group,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
