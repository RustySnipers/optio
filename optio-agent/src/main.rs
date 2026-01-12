use std::time::{Duration, SystemTime, UNIX_EPOCH};

use optio_core::proto::collector_client::CollectorClient;
use optio_core::proto::ScanResult;

#[tokio::main]
async fn main() {
    let mut client = match CollectorClient::connect("http://127.0.0.1:50051").await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("failed to connect to hub: {err}");
            return;
        }
    };

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();

        let scan_result = ScanResult {
            machine_id: "dummy-machine-id".to_string(),
            scan_id: format!("scan-{timestamp}"),
            target: "127.0.0.1".to_string(),
            started_at: timestamp.saturating_sub(30),
            finished_at: timestamp,
            status: "completed".to_string(),
            payload_json: r#"{"ports":[],"summary":"dummy"}"#.to_string(),
        };

        if let Err(err) = client.submit_scan_result(scan_result).await {
            eprintln!("failed to submit scan result: {err}");
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
