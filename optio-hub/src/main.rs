// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    // Shutdown signal for graceful gRPC server termination
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();

    // Start gRPC server in a background thread
    let grpc_handle = std::thread::spawn(move || {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => {
                eprintln!("Failed to create gRPC runtime: {err}");
                return;
            }
        };

        let addr = match "0.0.0.0:50051".parse() {
            Ok(addr) => addr,
            Err(err) => {
                eprintln!("Failed to parse gRPC address: {err}");
                return;
            }
        };

        runtime.block_on(async move {
            // Create a shutdown signal that watches the atomic flag
            let shutdown_signal = async move {
                loop {
                    if shutdown_flag_clone.load(Ordering::Relaxed) {
                        eprintln!("gRPC server received shutdown signal");
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            };

            if let Err(err) = optio_lib::grpc::start_grpc_server_with_shutdown(addr, shutdown_signal).await {
                eprintln!("gRPC server stopped: {err}");
            } else {
                eprintln!("gRPC server shut down gracefully");
            }
        });
    });

    // Run Tauri application (blocks until app closes)
    optio_lib::run();

    // Signal gRPC server to shut down
    eprintln!("Tauri app closed, signaling gRPC server to shut down...");
    shutdown_flag.store(true, Ordering::Relaxed);

    // Wait for gRPC server to finish (with timeout)
    let timeout = std::time::Duration::from_secs(5);
    let start = std::time::Instant::now();

    while !grpc_handle.is_finished() && start.elapsed() < timeout {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if grpc_handle.is_finished() {
        let _ = grpc_handle.join();
        eprintln!("gRPC server terminated successfully");
    } else {
        eprintln!("gRPC server did not terminate within timeout, exiting anyway");
    }
}
