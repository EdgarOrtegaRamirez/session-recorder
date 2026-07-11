use crate::models::{CommandEvent, Session, SessionSummary};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub enum StorageError {
    IoError(std::io::Error),
    SerializationError(String),
    NotFound(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(e) => write!(f, "IO error: {e}"),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {e}"),
            StorageError::NotFound(msg) => write!(f, "Not found: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::IoError(e)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::SerializationError(e.to_string())
    }
}

impl From<serde_yaml::Error> for StorageError {
    fn from(e: serde_yaml::Error) -> Self {
        StorageError::SerializationError(e.to_string())
    }
}

#[derive(Clone)]
pub struct SessionStorage {
    base_dir: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct CommandsFile {
    session_id: String,
    commands: Vec<CommandEvent>,
}

impl SessionStorage {
    pub fn new() -> Result<Self, StorageError> {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("session-recorder");

        fs::create_dir_all(base_dir.join("sessions"))?;
        fs::create_dir_all(base_dir.join("active"))?;

        Ok(Self { base_dir })
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.base_dir.join("sessions")
    }

    fn active_path(&self) -> PathBuf {
        self.base_dir.join("active").join("active.json")
    }

    fn commands_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir().join(format!("{session_id}.json"))
    }

    pub fn set_active_session(&self, session_id: &str, name: &str) -> Result<(), StorageError> {
        let active = Some((session_id.to_string(), name.to_string()));
        let json = serde_json::to_string_pretty(&active)?;

        let mut file = File::create(self.active_path())?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn get_active_session(&self) -> Result<(String, String), StorageError> {
        let active_path = self.active_path();
        if !active_path.exists() {
            return Err(StorageError::NotFound("No active session".to_string()));
        }

        let content = fs::read_to_string(&active_path)?;
        let session: Option<(String, String)> = serde_json::from_str(&content).map_err(|e| {
            StorageError::SerializationError(format!("Invalid active session: {e}"))
        })?;

        session.ok_or_else(|| StorageError::NotFound("No active session".to_string()))
    }

    pub fn clear_active_session(&self) -> Result<(), StorageError> {
        let active_path = self.active_path();
        if active_path.exists() {
            fs::remove_file(active_path)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn append_command(&self, session_id: &str, cmd: &CommandEvent) -> Result<(), StorageError> {
        let commands_path = self.commands_path(session_id);

        let mut commands_file = if commands_path.exists() {
            let content = fs::read_to_string(&commands_path)?;
            serde_json::from_str::<CommandsFile>(&content)?
        } else {
            CommandsFile {
                session_id: session_id.to_string(),
                commands: Vec::new(),
            }
        };

        commands_file.commands.push(cmd.clone());

        let content = serde_json::to_string_pretty(&commands_file)?;
        let mut file = File::create(&commands_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    pub fn get_session_commands(
        &self,
        session_id: &str,
    ) -> Result<Vec<CommandEvent>, StorageError> {
        let commands_path = self.commands_path(session_id);
        if !commands_path.exists() {
            return Err(StorageError::NotFound(format!(
                "Session {session_id} not found"
            )));
        }

        let content = fs::read_to_string(&commands_path)?;
        let commands_file: CommandsFile = serde_json::from_str(&content)?;

        Ok(commands_file.commands)
    }

    pub fn save_session(&self, session: &Session) -> Result<(), StorageError> {
        let content = serde_json::to_string_pretty(session)?;
        let mut file = File::create(self.commands_path(&session.id))?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn get_session(&self, session_id: &str, format: &str) -> Result<String, StorageError> {
        let commands_path = self.commands_path(session_id);
        if !commands_path.exists() {
            return Err(StorageError::NotFound(format!(
                "Session {session_id} not found"
            )));
        }

        let content = fs::read_to_string(&commands_path)?;
        let session: Session = serde_json::from_str(&content)?;

        match format.to_lowercase().as_str() {
            "json" => Ok(serde_json::to_string_pretty(&session)?),
            "yaml" => Ok(serde_yaml::to_string(&session)?),
            _ => Ok(self.format_text_session(&session)),
        }
    }

    fn format_text_session(&self, session: &Session) -> String {
        let mut output = String::new();
        output.push_str(&format!("Session: {}\n", session.name));
        output.push_str(&format!("ID: {}\n", session.id));
        output.push_str(&format!("Started: {}\n", session.start_time));
        output.push_str(&format!("Ended: {}\n", session.end_time));
        output.push_str(&format!(
            "Duration: {:.1}s\n",
            session.duration.as_secs_f64()
        ));
        output.push_str(&format!(
            "Host: {}@{}\n",
            session.username, session.hostname
        ));
        output.push_str(&format!("Commands: {}\n", session.commands.len()));
        output.push_str("\n--- Commands ---\n");

        for cmd in &session.commands {
            let first_line = cmd.stdout_preview.lines().next().unwrap_or("");
            output.push_str(&format!(
                "\n[{}] #{} exit={:3} {:.2}s\n  {}\n",
                cmd.sequence,
                cmd.command,
                cmd.exit_code,
                cmd.duration.as_secs_f64(),
                first_line
            ));
        }

        output
    }

    pub fn list_sessions(&self, format: &str) -> Result<Vec<SessionSummary>, StorageError> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut summaries = Vec::new();

        for entry in fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;
                let session: Session = match serde_json::from_str(&content) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                summaries.push(SessionSummary {
                    id: session.id.clone(),
                    name: session.name.clone(),
                    start_time: session.start_time.clone(),
                    command_count: session.commands.len(),
                    duration_seconds: session.duration.as_secs_f64(),
                    tag_count: session.tags.len(),
                });
            }
        }

        summaries.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        if format.to_lowercase().as_str() == "json" {
            println!("{}", serde_json::to_string_pretty(&summaries)?);
        } else if format.to_lowercase().as_str() == "yaml" {
            println!("{}", serde_yaml::to_string(&summaries)?);
        }

        Ok(summaries)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<(), StorageError> {
        let commands_path = self.commands_path(session_id);
        if !commands_path.exists() {
            return Err(StorageError::NotFound(format!(
                "Session {session_id} not found"
            )));
        }
        fs::remove_file(commands_path)?;
        Ok(())
    }
}
