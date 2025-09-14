# Copilot Instructions for `slugpm`

This project is a Rust CLI tool for managing project slugs and archiving files or directories. The codebase is intentionally minimal, with a focus on testability and clear workflows.

## Architecture & Key Concepts
- **Single-binary CLI**: Entrypoint is `src/main.rs`. Core logic is in `src/lib.rs` for modularity and testability.
- **Commands** (via `clap`):
  - `archive`: Move a file or directory to an `archive` folder (see below for rules).
  - `name`: Print the project name, stripping a leading date prefix.
  - Default (no subcommand): Create a new project directory under `project/<slug>` from a title (from args or piped stdin).
- **Slugification**: Uses the `slug` crate to create filesystem-safe names from titles.
- **Archiving rules**:
  - Files: Moved to `<parent>/archive/<filename>`.
  - Directories: Moved to `<parent>/../archive/<dirname>`.
  - If `-` is passed to `archive`, append stdin to the archive file instead of moving.
- **Testability**: All file system logic is abstracted via a `FileOps` trait. A `MockFileOps` is provided for in-memory, side-effect-free testing.

## Developer Workflows
- **Build**: `cargo build`
- **Run**: `cargo run -- [args]`
- **Test**: `cargo test` (tests live in `tests/integration.rs` and use the mock file system)
- **Dependencies**: Managed in `Cargo.toml`. Main crates: `anyhow`, `clap`, `atty`, `slug`, `regex`.

## Project Conventions
- **Modular logic**: CLI/command logic in `src/main.rs`, core logic in `src/lib.rs`.
- **Error handling**: Uses `anyhow::Result` for all main functions.
- **STDIN/STDOUT**: Many commands read from or write to standard streams. Detect piped input with `atty`.
- **Date prefix**: Project names may start with `YYYY-MM-DD-`; the `name` command strips this.
- **No config files**: All behavior is code-driven; no external config or environment variables.

## Examples
- Create a project: `echo 'My Project' | cargo run`
- Archive a file: `cargo run -- archive notes.txt`
- Archive a directory: `cargo run -- archive mydir/`
- Append to archive: `echo 'log' | cargo run -- archive notes.txt -`
- Print name: `cargo run -- name 2025-09-13-MyProject`

## Key Files
- `src/main.rs`: CLI and command logic.
- `src/lib.rs`: Core logic, traits, and testability.
- `tests/integration.rs`: Test suite using `MockFileOps`.
- `Cargo.toml`: Dependencies and metadata.

---
If you add new commands, change archiving logic, or update the test strategy, update this file with new patterns and examples.
