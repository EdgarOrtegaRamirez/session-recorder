use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents a single command execution event within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    pub sequence: u64,
    pub command: String,
    pub cwd: String,
    pub start_time: String,
    pub end_time: String,
    pub duration: Duration,
    pub exit_code: i32,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub stdout_preview: String,
    pub stderr_preview: String,
    /// Optional metadata tags added by the user
    pub tags: Vec<String>,
}

/// A recorded terminal session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
    pub duration: Duration,
    pub hostname: String,
    pub username: String,
    pub commands: Vec<CommandEvent>,
    pub tags: Vec<String>,
}

/// Summary information for listing sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: String,
    pub start_time: String,
    pub command_count: usize,
    pub duration_seconds: f64,
    pub tag_count: usize,
}

impl SessionSummary {
    pub fn short_display(&self) -> String {
        format!(
            "{}  {:<30}  {} commands  {:.1}s  [{}]",
            self.id, self.name, self.command_count, self.duration_seconds, self.tag_count
        )
    }
}
