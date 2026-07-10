# Session Recorder

Terminal session recorder and replayer with structured metadata. Record your terminal sessions, save them with timing and metadata, and replay them later.

## Features

- **Record** — Start/stop terminal sessions with automatic command tracking
- **Replay** — Replay recorded sessions with configurable speed and timing
- **Inspect** — View session details including commands, timing, exit codes, and output
- **Export** — Export sessions as JSON, YAML, or bash scripts
- **Structured storage** — Sessions stored as JSON with full metadata

## Install

```bash
# Build from source
cargo install --path .

# Or build and install locally
cargo build --release && cargo install --path .
```

## Quick Start

```bash
# Start recording
session-recorder start --name "my-session"

# ... do work in your terminal ...

# Stop recording
session-recorder stop

# List all sessions
session-recorder list

# View session details
session-recorder info <session-id>

# Replay a session
session-recorder replay <session-id> --speed 2.0

# Delete a session
session-recorder delete <session-id>
```

## Commands

| Command | Description |
|---------|-------------|
| `start` | Start recording a new session |
| `stop` | Stop the current recording session |
| `list` | List all recorded sessions |
| `replay` | Replay a recorded session |
| `info` | Show details of a session |
| `delete` | Delete a recorded session |

## Session Storage

Sessions are stored in `~/.local/share/session-recorder/sessions/` as individual JSON files. Each file contains the full session data including:

- Session metadata (ID, name, timestamps, hostname, username)
- Command list with timing, exit codes, and output
- Working directory for each command

## Architecture

```
session-recorder/
├── src/
│   ├── main.rs       # CLI entry point with clap
│   ├── models.rs     # Data models (Session, CommandEvent, SessionSummary)
│   ├── recorder.rs   # Session recording logic
│   ├── replayer.rs   # Session replay logic
│   └── storage.rs    # JSON file-based session storage
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
├── README.md
├── LICENSE
└── AGENTS.md
```

## Security

- Sessions are stored locally only
- No network communication
- Output is truncated to prevent sensitive data leakage (4KB limit)

## License

MIT
