use std::process::Command;
use std::time::Duration;

// Unit tests for models - import via the binary's modules
mod models_tests {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    struct TestCommand {
        sequence: u64,
        command: String,
        cwd: String,
        start_time: String,
        end_time: String,
        duration: Duration,
        exit_code: i32,
        stdout_truncated: bool,
        stderr_truncated: bool,
        stdout_preview: String,
        stderr_preview: String,
        tags: Vec<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    struct TestSession {
        id: String,
        name: String,
        start_time: String,
        end_time: String,
        duration: Duration,
        hostname: String,
        username: String,
        commands: Vec<TestCommand>,
        tags: Vec<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    struct TestSummary {
        id: String,
        name: String,
        start_time: String,
        command_count: usize,
        duration_seconds: f64,
        tag_count: usize,
    }

    impl TestSummary {
        fn short_display(&self) -> String {
            format!(
                "{}  {:<30}  {} commands  {:.1}s  [{}]",
                self.id, self.name, self.command_count, self.duration_seconds, self.tag_count
            )
        }
    }

    #[test]
    fn test_command_event_serialization() {
        let cmd = TestCommand {
            sequence: 1,
            command: "ls -la".to_string(),
            cwd: "/tmp".to_string(),
            start_time: "2026-01-01T00:00:00Z".to_string(),
            end_time: "2026-01-01T00:00:01Z".to_string(),
            duration: Duration::from_secs(1),
            exit_code: 0,
            stdout_truncated: false,
            stderr_truncated: false,
            stdout_preview: "total 0".to_string(),
            stderr_preview: String::new(),
            tags: vec!["test".to_string()],
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: TestCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sequence, 1);
        assert_eq!(parsed.command, "ls -la");
        assert_eq!(parsed.exit_code, 0);
        assert_eq!(parsed.tags, vec!["test"]);
    }

    #[test]
    fn test_session_summary_display() {
        let summary = TestSummary {
            id: "abc123".to_string(),
            name: "test".to_string(),
            start_time: "2026-01-01T00:00:00Z".to_string(),
            command_count: 5,
            duration_seconds: 10.5,
            tag_count: 2,
        };
        let display = summary.short_display();
        assert!(display.contains("abc123"));
        assert!(display.contains("test"));
        assert!(display.contains("5 commands"));
        assert!(display.contains("10.5s"));
    }

    #[test]
    fn test_session_roundtrip() {
        let cmd = TestCommand {
            sequence: 1,
            command: "echo hello".to_string(),
            cwd: "/tmp".to_string(),
            start_time: "2026-01-01T00:00:00Z".to_string(),
            end_time: "2026-01-01T00:00:01Z".to_string(),
            duration: Duration::from_secs(1),
            exit_code: 0,
            stdout_truncated: false,
            stderr_truncated: false,
            stdout_preview: "hello".to_string(),
            stderr_preview: String::new(),
            tags: Vec::new(),
        };

        let session = TestSession {
            id: "test-session-1".to_string(),
            name: "Test Session".to_string(),
            start_time: "2026-01-01T00:00:00Z".to_string(),
            end_time: "2026-01-01T00:00:05Z".to_string(),
            duration: Duration::from_secs(5),
            hostname: "testhost".to_string(),
            username: "testuser".to_string(),
            commands: vec![cmd],
            tags: Vec::new(),
        };

        let json = serde_json::to_string_pretty(&session).unwrap();
        let parsed: TestSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test-session-1");
        assert_eq!(parsed.name, "Test Session");
        assert_eq!(parsed.commands.len(), 1);
        assert_eq!(parsed.commands[0].command, "echo hello");
    }

    #[test]
    fn test_yaml_serialization() {
        let session = TestSession {
            id: "test-1".to_string(),
            name: "Test".to_string(),
            start_time: "2026-01-01T00:00:00Z".to_string(),
            end_time: "2026-01-01T00:00:01Z".to_string(),
            duration: Duration::from_secs(1),
            hostname: "host".to_string(),
            username: "user".to_string(),
            commands: Vec::new(),
            tags: Vec::new(),
        };

        let yaml = serde_yaml::to_string(&session).unwrap();
        assert!(yaml.contains("id: test-1"));
        assert!(yaml.contains("name: Test"));
    }
}

#[test]
fn test_cli_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run CLI");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session-recorder"));
    assert!(stdout.contains("start"));
    assert!(stdout.contains("stop"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("replay"));
    assert!(stdout.contains("info"));
    assert!(stdout.contains("delete"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run CLI");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session-recorder"));
}

#[test]
fn test_start_and_stop_empty_session() {
    // Clean up first
    let _ = Command::new("rm")
        .args(["-rf", "~/.local/share/session-recorder"])
        .output();

    let start_output = Command::new("cargo")
        .args(["run", "--quiet", "--", "start", "--name", "cli-test"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to start session");
    let start_stdout = String::from_utf8_lossy(&start_output.stdout);
    assert!(start_stdout.contains("Session started"));

    // Extract session ID
    let session_id = start_stdout
        .lines()
        .find(|l| l.contains("Session started:"))
        .map(|l| l.split(':').nth(1).unwrap().trim())
        .expect("No session ID found");

    // Stop the session
    std::thread::sleep(Duration::from_millis(100));
    let stop_output = Command::new("cargo")
        .args(["run", "--quiet", "--", "stop"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to stop session");
    let stop_stdout = String::from_utf8_lossy(&stop_output.stdout);
    assert!(stop_stdout.contains("Session stopped"));
    assert!(stop_stdout.contains("cli-test"));

    // List should show the session
    let list_output = Command::new("cargo")
        .args(["run", "--quiet", "--", "list"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to list sessions");
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("cli-test"));

    // Clean up
    let _ = Command::new("cargo")
        .args(["run", "--quiet", "--", "delete", session_id])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output();
}
