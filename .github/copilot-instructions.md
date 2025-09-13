# Copilot Instructions for `slugpm`

This project is a Rust CLI tool for managing project slugs and archiving files or directories. The codebase is intentionally minimal and focused on a few core workflows. Follow these guidelines to be productive as an AI coding agent in this repository.

## Architecture & Key Concepts
- **Single-binary CLI**: All logic is in `src/main.rs`. There are no modules or sub-crates.
- **Commands**: Uses `clap` for argument parsing. Main commands:
  - `archive`: Move a file or directory to an `archive` folder (see below for rules).
  - `name`: Print the project name, stripping a leading date prefix.
  - Default (no subcommand): Create a new project directory under `project/<slug>` from a title (from args or piped stdin).
- **Slugification**: Uses the `slug` crate to create filesystem-safe names from titles.
- **Archiving rules**:
  - Files: Moved to `<parent>/archive/<filename>`.
  - Directories: Moved to `<parent>/../archive/<dirname>`.
  - If `-` is passed to `archive`, append stdin to the archive file instead of moving.

## Developer Workflows
- **Build**: Use `cargo build` (no special scripts).
- **Run**: Use `cargo run -- [args]`.
- **Test**: No tests are present; add new ones in `src/main.rs` if needed.
- **Dependencies**: Managed in `Cargo.toml`. Uses only a few crates: `anyhow`, `clap`, `atty`, `slug`, `regex`.

## Project Conventions
- **No modules**: All logic is in a single file for simplicity.
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
- `src/main.rs`: All logic and entrypoint.
- `Cargo.toml`: Dependencies and metadata.

---
If you add new commands or change archiving logic, update this file with new patterns and examples.
