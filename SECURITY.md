# Session Recorder

Terminal session recorder and replayer with structured metadata.

## Security

This project has been reviewed for security best practices:

- **No network communication** — All data stored locally
- **No shell injection** — Commands are executed via `sh -c` with proper escaping
- **Output truncation** — 4KB limit prevents sensitive data leakage
- **No external dependencies** — Only standard library + well-known crates
