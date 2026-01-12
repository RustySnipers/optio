use optio_core::network::models::{ScanConfig, ScanType};
use optio_core::network::scanner::TcpScanner;

#[tokio::main]
async fn main() {
    let scan_config = ScanConfig {
        targets: vec!["127.0.0.1".to_string()],
        scan_type: ScanType::QuickScan,
        ..Default::default()
    };

    let scanner = TcpScanner::new();

    println!(
        "Starting {:?} for targets: {:?}",
        scan_config.scan_type, scan_config.targets
    );

    match scanner.scan_single_host("127.0.0.1").await {
        Ok(result) => {
            println!("Scan complete:");
            println!("{result:#?}");
        }
        Err(err) => {
            eprintln!("Scan failed: {err}");
        }
    }
}
