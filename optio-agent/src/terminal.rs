//! Interactive Terminal Handler
//!
//! Provides PTY-based interactive shell sessions for remote terminal access.
//! Uses the portable-pty crate for cross-platform pseudo-terminal support.

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Configuration for a terminal session
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    /// Shell type: "powershell", "cmd", "bash", "sh"
    pub shell_type: String,
    /// Initial terminal width in columns
    pub cols: u16,
    /// Initial terminal height in rows
    pub rows: u16,
    /// Working directory (optional)
    pub working_directory: Option<String>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            #[cfg(windows)]
            shell_type: "powershell".to_string(),
            #[cfg(not(windows))]
            shell_type: "bash".to_string(),
            cols: 80,
            rows: 24,
            working_directory: None,
        }
    }
}

/// Terminal session handle
pub struct TerminalSession {
    session_id: String,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child_pid: u32,
    shell_type: String,
}

impl TerminalSession {
    /// Start a new terminal session
    pub fn start(session_id: String, config: TerminalConfig) -> Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        // Open a pseudo-terminal pair
        let pair = pty_system
            .openpty(size)
            .context("Failed to open PTY pair")?;

        // Build the shell command
        let mut cmd = get_shell_command(&config.shell_type)?;

        // Set working directory if specified
        if let Some(ref cwd) = config.working_directory {
            cmd.cwd(cwd);
        }

        // Spawn the shell process
        let child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn shell process")?;

        let child_pid = child.process_id().unwrap_or(0);

        info!(
            session_id = %session_id,
            shell = %config.shell_type,
            pid = child_pid,
            "Terminal session started"
        );

        // Get the writer for sending input
        let writer = pair
            .master
            .take_writer()
            .context("Failed to get PTY writer")?;

        Ok(Self {
            session_id,
            master: pair.master,
            writer,
            child_pid,
            shell_type: config.shell_type,
        })
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the shell type
    pub fn shell_type(&self) -> &str {
        &self.shell_type
    }

    /// Get the child process ID
    pub fn pid(&self) -> u32 {
        self.child_pid
    }

    /// Write input to the terminal
    pub fn write_input(&mut self, data: &[u8]) -> Result<()> {
        self.writer
            .write_all(data)
            .context("Failed to write to terminal")?;
        self.writer.flush().context("Failed to flush terminal")?;
        Ok(())
    }

    /// Resize the terminal
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        self.master.resize(size).context("Failed to resize PTY")?;
        debug!(
            session_id = %self.session_id,
            cols = cols,
            rows = rows,
            "Terminal resized"
        );
        Ok(())
    }

    /// Start reading output from the terminal
    /// Returns a channel receiver for terminal output
    pub fn start_output_reader(
        &self,
    ) -> Result<mpsc::Receiver<TerminalOutputEvent>> {
        let (tx, rx) = mpsc::channel(1024);

        let mut reader = self
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;

        let session_id = self.session_id.clone();

        // Spawn a thread to read from the PTY
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - terminal closed
                        debug!(session_id = %session_id, "Terminal EOF");
                        let _ = tx.blocking_send(TerminalOutputEvent::Ended {
                            exit_code: 0,
                            reason: "EOF".to_string(),
                        });
                        break;
                    }
                    Ok(n) => {
                        let data = buffer[..n].to_vec();
                        if tx.blocking_send(TerminalOutputEvent::Data(data)).is_err() {
                            debug!(session_id = %session_id, "Output channel closed");
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        error!(
                            session_id = %session_id,
                            error = %e,
                            "Error reading from terminal"
                        );
                        let _ = tx.blocking_send(TerminalOutputEvent::Error {
                            code: "READ_ERROR".to_string(),
                            message: e.to_string(),
                        });
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

/// Events emitted by the terminal output reader
#[derive(Debug)]
pub enum TerminalOutputEvent {
    /// Data read from the terminal
    Data(Vec<u8>),
    /// Terminal session ended
    Ended { exit_code: i32, reason: String },
    /// Error occurred
    Error { code: String, message: String },
}

/// Get the shell command based on shell type
fn get_shell_command(shell_type: &str) -> Result<CommandBuilder> {
    let mut cmd = match shell_type.to_lowercase().as_str() {
        "powershell" => {
            // Log CLM status for security awareness
            log_powershell_language_mode();
            
            let mut c = CommandBuilder::new("powershell.exe");
            c.args(["-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass"]);
            c
        }
        "cmd" => CommandBuilder::new("cmd.exe"),
        "bash" => CommandBuilder::new("bash"),
        "sh" => CommandBuilder::new("sh"),
        #[cfg(windows)]
        _ => {
            // Default to PowerShell on Windows
            log_powershell_language_mode();
            
            let mut c = CommandBuilder::new("powershell.exe");
            c.args(["-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass"]);
            c
        }
        #[cfg(not(windows))]
        _ => {
            // Default to bash on Unix
            CommandBuilder::new("bash")
        }
    };

    Ok(cmd)
}

/// Check and log PowerShell Constrained Language Mode (CLM) status
///
/// CLM is a security feature in PowerShell that restricts available language 
/// elements to limit attack surface. When CLM is active:
/// - .NET types are restricted
/// - Add-Type and scripts are limited
/// - Only core cmdlets are available
///
/// For the prototype, we log the mode for awareness. In production, this
/// could gate certain functionality.
#[cfg(windows)]
fn log_powershell_language_mode() {
    use std::process::Command;
    
    // Query PowerShell language mode
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$ExecutionContext.SessionState.LanguageMode"
        ])
        .output();
    
    match output {
        Ok(output) if output.status.success() => {
            let mode = String::from_utf8_lossy(&output.stdout).trim().to_string();
            match mode.as_str() {
                "FullLanguage" => {
                    info!("PowerShell running in FullLanguage mode (unrestricted)");
                }
                "ConstrainedLanguage" => {
                    warn!(
                        "PowerShell running in ConstrainedLanguage mode (CLM). \
                         Some features may be restricted. This is typically due to \
                         AppLocker, WDAC, or DeviceGuard policies."
                    );
                }
                "RestrictedLanguage" => {
                    warn!(
                        "PowerShell running in RestrictedLanguage mode. \
                         Script execution is heavily restricted."
                    );
                }
                "NoLanguage" => {
                    warn!("PowerShell running in NoLanguage mode. Scripts cannot run.");
                }
                _ => {
                    debug!("PowerShell language mode: {}", mode);
                }
            }
        }
        Ok(output) => {
            debug!(
                "Could not determine PowerShell language mode: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            debug!("Failed to query PowerShell language mode: {}", e);
        }
    }
}

#[cfg(not(windows))]
fn log_powershell_language_mode() {
    // No-op on non-Windows platforms
}

/// Terminal session manager
pub struct TerminalManager {
    sessions: std::sync::Mutex<std::collections::HashMap<String, TerminalSession>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Start a new terminal session
    pub fn start_session(&self, session_id: String, config: TerminalConfig) -> Result<()> {
        let session = TerminalSession::start(session_id.clone(), config)?;

        let mut sessions = self.sessions.lock().map_err(|e| {
            anyhow::anyhow!("Failed to lock sessions: {}", e)
        })?;

        sessions.insert(session_id, session);
        Ok(())
    }

    /// Get a reference to a session
    pub fn with_session<F, R>(&self, session_id: &str, f: F) -> Result<R>
    where
        F: FnOnce(&mut TerminalSession) -> Result<R>,
    {
        let mut sessions = self.sessions.lock().map_err(|e| {
            anyhow::anyhow!("Failed to lock sessions: {}", e)
        })?;

        let session = sessions.get_mut(session_id).ok_or_else(|| {
            anyhow::anyhow!("Session not found: {}", session_id)
        })?;

        f(session)
    }

    /// Remove a session
    pub fn remove_session(&self, session_id: &str) -> Option<TerminalSession> {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(session_id)
        } else {
            None
        }
    }

    /// Check if a session exists
    pub fn has_session(&self, session_id: &str) -> bool {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.contains_key(session_id)
        } else {
            false
        }
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.len()
        } else {
            0
        }
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_config_default() {
        let config = TerminalConfig::default();
        assert_eq!(config.cols, 80);
        assert_eq!(config.rows, 24);
        assert!(config.working_directory.is_none());
    }

    #[test]
    fn test_terminal_manager() {
        let manager = TerminalManager::new();
        assert_eq!(manager.session_count(), 0);
        assert!(!manager.has_session("test"));
    }
}
