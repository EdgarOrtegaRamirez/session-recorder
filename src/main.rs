mod models;
mod recorder;
mod replayer;
mod storage;

use clap::Parser;
use recorder::SessionRecorder;
use storage::SessionStorage;

#[derive(Parser, Debug)]
#[command(name = "session-recorder")]
#[command(about = "Terminal session recorder and replayer with structured metadata")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Start recording a new session
    Start {
        /// Optional name for the session
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Stop the current recording session
    Stop,
    /// List all recorded sessions
    List {
        /// Output format: text, json, yaml
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Replay a recorded session
    Replay {
        /// Session ID to replay
        session_id: String,
        /// Speed multiplier (1.0 = real-time)
        #[arg(short, long, default_value = "1.0")]
        speed: f64,
        /// Skip prompts (only show commands)
        #[arg(long)]
        no_interact: bool,
    },
    /// Show details of a session
    Info {
        /// Session ID to show
        session_id: String,
        /// Output format: text, json
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Delete a recorded session
    Delete {
        /// Session ID to delete
        session_id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let storage = SessionStorage::new().expect("Failed to initialize storage");
    let recorder = SessionRecorder::new(storage.clone());

    match cli.command {
        Command::Start { name } => match recorder.start_session(name) {
            Ok(session_id) => {
                println!("Session started: {session_id}");
                println!("Recording commands and timing...");
                println!("Run 'session-recorder stop' to finish");
            }
            Err(e) => eprintln!("Error starting session: {e}"),
        },
        Command::Stop => match recorder.stop_session() {
            Ok(summary) => {
                println!("Session stopped.");
                println!("{summary}");
            }
            Err(e) => eprintln!("Error stopping session: {e}"),
        },
        Command::List { format } => match storage.list_sessions(&format) {
            Ok(sessions) => {
                if sessions.is_empty() {
                    println!("No recorded sessions found.");
                    return;
                }
                for s in &sessions {
                    println!("{}", s.short_display());
                }
            }
            Err(e) => eprintln!("Error listing sessions: {e}"),
        },
        Command::Replay {
            session_id,
            speed,
            no_interact,
        } => match replayer::replay_session(&storage, &session_id, speed, no_interact) {
            Ok(_) => println!("\nReplay complete."),
            Err(e) => eprintln!("Error replaying session: {e}"),
        },
        Command::Info { session_id, format } => match storage.get_session(&session_id, &format) {
            Ok(info) => println!("{info}"),
            Err(e) => eprintln!("Error getting session info: {e}"),
        },
        Command::Delete { session_id } => match storage.delete_session(&session_id) {
            Ok(_) => println!("Session deleted: {session_id}"),
            Err(e) => eprintln!("Error deleting session: {e}"),
        },
    }
}
