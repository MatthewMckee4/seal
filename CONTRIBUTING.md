# Contributing to Seal

Welcome, and thanks for contributing to Seal.

## Finding Ways to Help

[`good first issue`](https://github.com/MatthewMckee4/seal/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22)
issues are ready for new contributors.
[`help wanted`](https://github.com/MatthewMckee4/seal/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22)
issues are good candidates when you already know the codebase.

Comment before starting work so another contributor does not duplicate it and
the maintainer can confirm the issue is current. Discuss larger changes and
new features before opening a pull request; they can affect Seal's scope and
long-term maintenance burden.

Use [GitHub issues](https://github.com/MatthewMckee4/seal/issues/new) for bug
reports, feature proposals, and documentation problems.

## The Basics

### Prerequisites

Seal development requires the
[Rust toolchain](https://www.rust-lang.org/tools/install). Rustup will use the
repository's pinned toolchain.

We recommend [nextest](https://nexte.st/) for faster Rust test runs:

```sh
cargo install cargo-nextest --locked
```

Install the optional repository hooks with:

```sh
uvx prek install
```

### Development

Build the workspace:

```sh
cargo build --workspace
```

Run Seal from the repository root with a debug build:

```sh
cargo run -p seal -- --help
cargo run -p seal -- self version
```

### Project Structure

The `seal` binary parses CLI arguments, resolves project configuration, and
delegates work to focused library crates. All Rust crates live under
`crates/`:

- `seal` contains the CLI entry point, command handlers, global settings, and
  user-facing output.
- `seal_cli` contains shared Clap argument and command definitions.
- `seal_project` handles `seal.toml` parsing, validation, project discovery,
  and workspace members.
- `seal_bump` calculates semantic versions and updates version files.
- `seal_changelog` generates changelogs and release metadata.
- `seal_github` provides GitHub API access and repository remote parsing.
- `seal_file_change` handles file-change previews, diffs, and application.
- `seal_command` runs Git and configured subprocesses.
- `seal_fs`, `seal_logging`, `seal_terminal`, and `seal_version` provide shared
  infrastructure.
- `seal_options_metadata` and `seal_macros` support generated configuration
  metadata.
- `seal_dev` contains documentation generators.

Documentation lives under `docs/`, and CLI integration tests live under
`crates/seal/tests/it/`.

Command handlers send normal output through `Printer`. Diagnostics controlled
by `-v` use `tracing`.

## Testing

Run the full suite:

```sh
cargo nextest run --all-features
```

Pass standard `cargo nextest` arguments for focused runs:

```sh
cargo nextest run -p seal
cargo nextest run -p seal test_name
```

Use `cargo test` with the same arguments when nextest is unavailable.

Before opening a pull request, run relevant focused tests and the validation
sweep:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
uvx prek run -a
```

GitHub Actions runs tests on Linux, macOS, and Windows. Platform-specific
behavior should have focused coverage where practical.

### Snapshot Tests

Prefer integration tests when behavior crosses crate or CLI boundaries.
Command integration tests should use the existing `seal_snapshot!` helper so
diagnostics and exit behavior remain visible.

Review every snapshot change before accepting it. Check for unexpected
`.snap.new` files before finishing, and never include unrelated snapshot
updates in a pull request.

## Documentation

Seal uses [Zensical](https://zensical.org/) for its documentation site. Prepare
the generated home page and build the site with:

```sh
uv run --script scripts/prepare_docs.py
uv run --isolated --with-requirements docs/requirements.txt zensical build
```

Run the documentation generator after changing configuration options, CLI
arguments, or their source documentation:

```sh
cargo dev generate-all
```

Files under `docs/reference/` are generated. Do not edit them manually; review
the generated diff before committing it.

## Opening a Pull Request

Use the pull request template, link relevant issues, and add labels that match
the affected area. Keep the pull request in draft while substantial work
remains.

### Summary

Explain what changed and why in concise prose. Include implementation details
only when reviewers need them to understand the design or trade-offs.

### Test Plan

State what you verified in one short sentence. If CI is the only remaining
validation, write `ci`.

Keep commits focused and use descriptive one-line subjects. Do not mix
formatter churn or unrelated cleanup with the change.

## Release Process

Run the `Prepare release` workflow with the version bump to perform, such as
`alpha` or an explicit version. The workflow runs `seal bump <version>` and
opens the release pull request.

To prepare a release locally, run:

```sh
cargo run -p seal -- bump <version>
```

Seal creates the release branch, commits and pushes the changes, and opens a
pull request. Review and merge it, then run the
[release workflow](https://github.com/MatthewMckee4/seal/actions/workflows/release.yml)
with the version tag without a leading `v`; it builds artifacts, creates the
GitHub release, and publishes the documentation.

## GitHub Actions

Actions must be pinned to full commit SHAs. After editing a workflow, run:

```sh
pinact run
```

Review generated workflow changes before committing them.
