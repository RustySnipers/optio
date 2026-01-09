//! Optio - Consultant-in-a-Box Backend
//!
//! High-performance, local-first security toolkit for Enterprise Architects
//! and IT Security Consultants.

pub mod commands;
pub mod factory;
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
