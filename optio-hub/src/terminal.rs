//! Terminal Proxy for Interactive Remote Shell Sessions
//!
//! This module bridges the frontend (via Tauri events) to agent terminal sessions
//! using gRPC streaming.

use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use optio_core::proto::terminal_service_client::TerminalServiceClient;
use optio_core::proto::{
    TerminalClose, TerminalData, TerminalInput, TerminalOutput, TerminalResize,
    TerminalStartRequest,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex, RwLock};
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Terminal session state
struct TerminalSessionState {
    /// Channel to send input to the gRPC stream
    input_tx: mpsc::Sender<TerminalInput>,
    /// Agent ID this session is connected to
    agent_id: String,
}

/// Terminal manager for handling multiple sessions
pub struct TerminalManager {
    /// Active terminal sessions
    sessions: RwLock<HashMap<String, TerminalSessionState>>,
    /// App handle for emitting events
    app_handle: Option<AppHandle>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            app_handle: None,
        }
    }

    pub fn with_app_handle(mut self, app_handle: AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    /// Start a new terminal session with an agent
    pub async fn start_session(
        &self,
        agent_id: String,
        agent_address: String,
        shell_type: Option<String>,
        cols: Option<u32>,
        rows: Option<u32>,
    ) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();

        info!(
            session_id = %session_id,
            agent_id = %agent_id,
            address = %agent_address,
            "Starting terminal session"
        );

        // Connect to the agent's terminal service
        let channel = Channel::from_shared(agent_address.clone())
            .context("Invalid agent address")?
            .connect()
            .await
            .context("Failed to connect to agent")?;

        let mut client = TerminalServiceClient::new(channel);

        // Create input channel
        let (input_tx, input_rx) = mpsc::channel::<TerminalInput>(256);
        let input_stream = tokio_stream::wrappers::ReceiverStream::new(input_rx);

        // Start the terminal RPC
        let response = client
            .start_terminal(input_stream)
            .await
            .context("Failed to start terminal RPC")?;

        let mut output_stream = response.into_inner();

        // Send the start request
        let start_input = TerminalInput {
            session_id: session_id.clone(),
            machine_id: agent_id.clone(),
            input: Some(optio_core::proto::terminal_input::Input::Start(
                TerminalStartRequest {
                    shell_type: shell_type.unwrap_or_default(),
                    cols: cols.unwrap_or(80),
                    rows: rows.unwrap_or(24),
                    working_directory: String::new(),
                    environment_json: String::new(),
                },
            )),
        };

        input_tx
            .send(start_input)
            .await
            .context("Failed to send start request")?;

        // Store the session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(
                session_id.clone(),
                TerminalSessionState {
                    input_tx,
                    agent_id: agent_id.clone(),
                },
            );
        }

        // Spawn a task to read output and emit events
        let app_handle = self.app_handle.clone();
        let sid = session_id.clone();
        let sessions_ref = Arc::new(&self.sessions);

        tokio::spawn(async move {
            while let Some(output_result) = output_stream.next().await {
                match output_result {
                    Ok(output) => {
                        if let Some(ref handle) = app_handle {
                            Self::emit_terminal_event(handle, &output);
                        }

                        // Check if session ended
                        if let Some(optio_core::proto::terminal_output::Output::Ended(_)) =
                            output.output
                        {
                            debug!(session_id = %sid, "Terminal session ended from agent");
                            break;
                        }
                    }
                    Err(e) => {
                        error!(session_id = %sid, error = %e, "Error receiving terminal output");
                        if let Some(ref handle) = app_handle {
                            let _ = handle.emit(
                                &format!("terminal:error:{}", sid),
                                serde_json::json!({
                                    "session_id": sid,
                                    "error": e.to_string()
                                }),
                            );
                        }
                        break;
                    }
                }
            }

            info!(session_id = %sid, "Terminal output stream ended");
        });

        Ok(session_id)
    }

    /// Send input data to a terminal session
    pub async fn send_input(&self, session_id: &str, data: Vec<u8>) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let input = TerminalInput {
            session_id: session_id.to_string(),
            machine_id: session.agent_id.clone(),
            input: Some(optio_core::proto::terminal_input::Input::Data(TerminalData {
                data,
            })),
        };

        session
            .input_tx
            .send(input)
            .await
            .context("Failed to send input")?;

        Ok(())
    }

    /// Resize a terminal session
    pub async fn resize(&self, session_id: &str, cols: u32, rows: u32) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let input = TerminalInput {
            session_id: session_id.to_string(),
            machine_id: session.agent_id.clone(),
            input: Some(optio_core::proto::terminal_input::Input::Resize(
                TerminalResize { cols, rows },
            )),
        };

        session
            .input_tx
            .send(input)
            .await
            .context("Failed to send resize")?;

        debug!(session_id = %session_id, cols = cols, rows = rows, "Terminal resized");
        Ok(())
    }

    /// Close a terminal session
    pub async fn close_session(&self, session_id: &str, reason: Option<String>) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.remove(session_id) {
            let input = TerminalInput {
                session_id: session_id.to_string(),
                machine_id: session.agent_id.clone(),
                input: Some(optio_core::proto::terminal_input::Input::Close(
                    TerminalClose {
                        reason: reason.unwrap_or_else(|| "User closed".to_string()),
                    },
                )),
            };

            // Try to send close, but don't fail if the channel is closed
            let _ = session.input_tx.send(input).await;

            info!(session_id = %session_id, "Terminal session closed");
        }

        Ok(())
    }

    /// Check if a session exists
    pub async fn has_session(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(session_id)
    }

    /// Get the number of active sessions
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Emit a terminal event to the frontend
    fn emit_terminal_event(app_handle: &AppHandle, output: &TerminalOutput) {
        let session_id = &output.session_id;

        match &output.output {
            Some(optio_core::proto::terminal_output::Output::Started(started)) => {
                let _ = app_handle.emit(
                    &format!("terminal:started:{}", session_id),
                    serde_json::json!({
                        "session_id": session_id,
                        "shell_type": started.shell_type,
                        "pid": started.pid
                    }),
                );
            }
            Some(optio_core::proto::terminal_output::Output::Data(data)) => {
                let _ = app_handle.emit(
                    &format!("terminal:data:{}", session_id),
                    serde_json::json!({
                        "session_id": session_id,
                        "data": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data.data)
                    }),
                );
            }
            Some(optio_core::proto::terminal_output::Output::Ended(ended)) => {
                let _ = app_handle.emit(
                    &format!("terminal:ended:{}", session_id),
                    serde_json::json!({
                        "session_id": session_id,
                        "exit_code": ended.exit_code,
                        "reason": ended.reason
                    }),
                );
            }
            Some(optio_core::proto::terminal_output::Output::Error(error)) => {
                let _ = app_handle.emit(
                    &format!("terminal:error:{}", session_id),
                    serde_json::json!({
                        "session_id": session_id,
                        "code": error.code,
                        "message": error.message
                    }),
                );
            }
            None => {}
        }
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared terminal state for Tauri
pub type TerminalState = Arc<Mutex<TerminalManager>>;

/// Create a new terminal state
pub fn create_terminal_state() -> TerminalState {
    Arc::new(Mutex::new(TerminalManager::new()))
}
