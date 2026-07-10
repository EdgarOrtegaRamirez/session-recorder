# AGENTS.md

## Project Overview
**Session Recorder** is a CLI tool for recording and replaying terminal sessions with structured metadata. It captures commands, timing, exit codes, and output for later replay or analysis.

## Tech Stack
- **Language:** Rust 2021
- **CLI Framework:** clap 4.5 (derive)
- **Serialization:** serde + serde_json + serde_yaml
- **UUID:** uuid v1.10 (v4)
- **Time:** chrono 0.4
- **Storage:** JSON files in `~/.local/share/session-recorder/`

## Project Structure
```
session-recorder/
├── src/
│   ├── main.rs       # CLI entry point, clap subcommands
│   ├── models.rs     # Session, CommandEvent, SessionSummary
│   ├── recorder.rs   # SessionRecorder, execute_command
│   ├── replayer.rs   # replay_session, format_as_script
│   └── storage.rs    # SessionStorage (JSON file-based)
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
├── README.md
├── LICENSE
└── AGENTS.md
```

## Build & Test
```bash
cargo build
cargo test
cargo run -- --help
```

## Key Design Decisions
1. **JSON file storage** — Simple, human-readable, no database dependency
2. **CLI-first** — Designed as a terminal tool, not a library
3. **No PTY dependency** — Records via command execution, not terminal emulation
4. **Output truncation** — 4KB limit per command to prevent storage bloat

## Common Pitfalls
- **Session files are per-session** — Each session gets its own JSON file in `sessions/`
- **Active session tracking** — Uses a separate `active/` directory for the current recording session
- **No PTY needed** — Commands are executed via `sh -c`, not terminal capture
