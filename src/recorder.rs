use crate::models::{CommandEvent, Session};
use crate::storage::{SessionStorage, StorageError};
use std::process::Command;
use std::time::{Duration, Instant};

pub struct SessionRecorder {
    storage: SessionStorage,
}

#[derive(Debug)]
pub enum RecorderError {
    NoActiveSession,
    #[allow(dead_code)]
    AlreadyActive,
    IoError(std::io::Error),
    SerializationError(String),
}

impl std::fmt::Display for RecorderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecorderError::NoActiveSession => write!(f, "No active recording session"),
            RecorderError::AlreadyActive => write!(f, "A session is already being recorded"),
            RecorderError::IoError(e) => write!(f, "IO error: {e}"),
            RecorderError::SerializationError(e) => write!(f, "Serialization error: {e}"),
        }
    }
}

impl std::error::Error for RecorderError {}

impl From<std::io::Error> for RecorderError {
    fn from(e: std::io::Error) -> Self {
        RecorderError::IoError(e)
    }
}

impl From<StorageError> for RecorderError {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::IoError(io) => RecorderError::IoError(io),
            StorageError::SerializationError(msg) => RecorderError::SerializationError(msg),
            StorageError::NotFound(msg) => RecorderError::SerializationError(msg),
        }
    }
}

impl SessionRecorder {
    pub fn new(storage: SessionStorage) -> Self {
        Self { storage }
    }

    pub fn start_session(&self, name: Option<String>) -> Result<String, RecorderError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let name = name.unwrap_or_else(|| {
            chrono::Utc::now()
                .format("session-%Y%m%d-%H%M%S")
                .to_string()
        });

        self.storage
            .set_active_session(&session_id, &name)
            .map_err(|e| RecorderError::SerializationError(e.to_string()))?;

        Ok(session_id)
    }

    pub fn stop_session(&self) -> Result<String, RecorderError> {
        let (session_id, name) = self
            .storage
            .get_active_session()
            .map_err(|_| RecorderError::NoActiveSession)?;

        let commands = self
            .storage
            .get_session_commands(&session_id)
            .unwrap_or_default();
        let now = chrono::Utc::now();
        let start_time = commands
            .first()
            .map(|c| c.start_time.clone())
            .unwrap_or_else(|| now.to_rfc3339());

        let duration = if let Some(last) = commands.last() {
            let start = chrono::DateTime::parse_from_rfc3339(&start_time)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let end = chrono::DateTime::parse_from_rfc3339(&last.end_time)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc));
            match (start, end) {
                (Some(s), Some(e)) => (e - s).to_std().unwrap_or(Duration::ZERO),
                _ => Duration::ZERO,
            }
        } else {
            Duration::ZERO
        };

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let username = whoami::username();

        let session = Session {
            id: session_id.clone(),
            name,
            start_time,
            end_time: now.to_rfc3339(),
            duration,
            hostname,
            username,
            commands,
            tags: Vec::new(),
        };

        self.storage.save_session(&session)?;
        self.storage.clear_active_session()?;

        Ok(format!(
            "Session '{}' ({}) — {} commands, {:.1}s total",
            session.name,
            session.id,
            session.commands.len(),
            session.duration.as_secs_f64()
        ))
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn record_command(
        &self,
        session_id: &str,
        command: &str,
        cwd: &str,
        exit_code: i32,
        stdout: &str,
        stderr: &str,
        duration: Duration,
    ) -> Result<(), RecorderError> {
        let now = chrono::Utc::now();
        let commands = self.storage.get_session_commands(session_id)?;
        let sequence = commands.len() as u64 + 1;

        let cmd = CommandEvent {
            sequence,
            command: command.to_string(),
            cwd: cwd.to_string(),
            start_time: now.to_rfc3339(),
            end_time: now.to_rfc3339(),
            duration,
            exit_code,
            stdout_truncated: stdout.len() > 4096,
            stderr_truncated: stderr.len() > 4096,
            stdout_preview: truncate_output(stdout, 4096),
            stderr_preview: truncate_output(stderr, 4096),
            tags: Vec::new(),
        };

        self.storage.append_command(session_id, &cmd)?;
        Ok(())
    }
}

#[allow(dead_code)]
fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        let truncated: String = output.chars().take(max_len).collect();
        format!("{truncated}\n... [truncated, {} total bytes]", output.len())
    }
}

/// Execute a command with timing and output capture
#[allow(dead_code)]
pub fn execute_command(command: &str, cwd: &str) -> (i32, String, String, Duration) {
    let start = Instant::now();

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .output();

    let duration = start.elapsed();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            (out.status.code().unwrap_or(-1), stdout, stderr, duration)
        }
        Err(e) => {
            let stderr = format!("Failed to execute: {e}");
            (-1, String::new(), stderr, duration)
        }
    }
}
