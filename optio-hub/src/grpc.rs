//! gRPC server for receiving scan results from agents.

use optio_core::proto::collector_server::{Collector, CollectorServer};
use optio_core::proto::{ScanResult, SubmitScanResultResponse};
use std::net::SocketAddr;
use tonic::{Request, Response, Status};
use tonic::transport::Server;

#[derive(Default)]
pub struct CollectorService;

#[tonic::async_trait]
impl Collector for CollectorService {
    async fn submit_scan_result(
        &self,
        request: Request<ScanResult>,
    ) -> Result<Response<SubmitScanResultResponse>, Status> {
        let scan = request.into_inner();

        tracing::info!(
            machine_id = %scan.machine_id,
            scan_id = %scan.scan_id,
            target = %scan.target,
            status = %scan.status,
            "Received scan result"
        );

        let response = SubmitScanResultResponse {
            accepted: true,
            message: "Scan result received".to_string(),
        };

        Ok(Response::new(response))
    }
}

pub async fn start_grpc_server(addr: SocketAddr) -> Result<(), tonic::transport::Error> {
    Server::builder()
        .add_service(CollectorServer::new(CollectorService::default()))
        .serve(addr)
        .await
}
