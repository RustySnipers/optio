use std::time::{Duration, SystemTime, UNIX_EPOCH};

use optio_core::proto::heartbeat::Status;
use optio_core::proto::heartbeat_service_client::HeartbeatServiceClient;
use optio_core::proto::Heartbeat;

#[tokio::main]
async fn main() {
    let mut client = match HeartbeatServiceClient::connect("http://127.0.0.1:50051").await {
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

        let heartbeat = Heartbeat {
            machine_id: "dummy-machine-id".to_string(),
            hostname: "dummy-host".to_string(),
            os_info: "dummy-os".to_string(),
            timestamp,
            status: Status::Online as i32,
        };

        if let Err(err) = client.send_heartbeat(heartbeat).await {
            eprintln!("failed to send heartbeat: {err}");
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
