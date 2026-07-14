# Style Guide

This guide covers user-facing text in Seal documentation, CLI output, issue templates, and release
notes.

## General

- Write `Seal` for the project and `seal` for the executable and configuration file prefix.
- Use direct, concrete language. Prefer "run `seal bump patch`" over "it is possible to bump the
  patch version".
- Use backticks for commands, flags, environment variables, file paths, configuration keys, crate
  names, and code expressions.
- Avoid bare URLs in prose. Prefer descriptive links.
- Wrap Markdown at 100 characters unless the file is generated.
- Add language tags to fenced code blocks.
- Use "release management", "version bump", and "changelog" consistently.

## Documentation

- Start with the user's task, then add details and edge cases.
- Keep generated reference pages generated. Update the Rust source and run
  `cargo dev generate-all` instead of editing them.
- Use `console` fences for commands and their output. Use `sh` or `powershell` for shell scripts.
- Include output only when its exact contents matter.
- Link guides to the complete CLI or configuration reference instead of duplicating every option.

## CLI Output

- Errors should state what failed and include the relevant file, configuration key, flag, or
  command.
- Output must remain understandable without color.
- Write machine-readable results to stdout. Write diagnostics and warnings to stderr.
- Send command output through `Printer` so `--quiet`, `--verbose`, and `--no-progress` remain
  consistent.

## Terminology

- Use "pre-release" for a semantic version before its stable release.
- Use "release branch" for the branch configured by `release.branch-name`.
- Use "version file" for a file selected by `release.version-files`.
- Use "workspace member" for a project listed under `[members]` in `seal.toml`.
- Use "snapshot", not "golden file", for Insta output.
