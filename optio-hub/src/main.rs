// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    std::thread::spawn(|| {
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
            if let Err(err) = optio_lib::grpc::start_grpc_server(addr).await {
                eprintln!("gRPC server stopped: {err}");
            }
        });
    });

    optio_lib::run()
}
