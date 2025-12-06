# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Seal is a release management tool written in Rust (early development). Cargo workspace with main CLI in `crates/seal` (seal package).

## Style

Do not Add too many comments, only add comments where necessary, if the code is complicated and cannot be simplified.

## Development Commands

```bash
# Build and test
cargo build
cargo test

# Format
cargo fmt

# Run CLI
cargo run -p seal_cli -- <args>
```

## Architecture

### Output System

**Critical**: Never use `print!` or `eprintln!` directly. All output goes through the `Printer` abstraction in `printer.rs`, configured from global CLI flags (`-q`, `-v`, `--no-progress`) via `GlobalSettings` in `settings.rs`.

### CLI Flow

1. `main.rs` parses args with clap → creates `Printer` from `GlobalSettings`
1. Routes to command handlers in `src/commands/`
1. Returns `ExitStatus` enum (Success=0, Failure=1, Error=2, External)

### Command Functions

**Critical**: All command functions MUST return `Result<ExitStatus>` from `anyhow`. Use `?` to propagate errors - they will be automatically displayed to the user by the error handling in `main.rs`.

## Code Conventions

- **Edition 2024, MSRV 1.89**
- **Strict clippy pedantic** - notably `print_stdout`/`print_stderr`/`dbg_macro` are forbidden
- **Use `Printer`** instead of direct print statements
