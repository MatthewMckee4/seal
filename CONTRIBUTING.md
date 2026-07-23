# Contributing to Seal

Contributions of all kinds are welcome. Open an
[issue](https://github.com/MatthewMckee4/seal/issues/new) for bugs, feature ideas, or documentation
improvements.

Small fixes can go straight to a pull request. For larger changes, open or comment on an issue
first so the intended behavior is clear before implementation starts.

Issues suitable for a first contribution are labelled
[`good first issue`](https://github.com/MatthewMckee4/seal/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22).
Issues where help is especially useful are labelled
[`help wanted`](https://github.com/MatthewMckee4/seal/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22).

## Architecture

Seal is a Cargo workspace. The `seal` binary parses CLI arguments, resolves project configuration,
and delegates version, changelog, GitHub, and file operations to focused library crates.

The executable and core domain crates are:

- `seal` — CLI entry point, command handlers, global settings, and user-facing output.
- `seal_cli` — shared Clap argument and command definitions.
- `seal_project` — `seal.toml` parsing, validation, project discovery, and workspace members.
- `seal_bump` — semantic-version calculation and version-file updates.
- `seal_changelog` — changelog generation, updates, and release-body metadata.
- `seal_github` — GitHub API access and repository remote parsing.
- `seal_file_change` — file-change previews, diffs, and application.
- `seal_command` — Git and configured subprocess execution.

Supporting crates are:

- `seal_fs` — paths relative to the project root.
- `seal_logging` — tracing output formatting.
- `seal_terminal` — terminal sizing.
- `seal_version` — the released CLI version.
- `seal_options_metadata` and `seal_macros` — metadata used to generate the configuration reference.
- `seal_dev` — developer commands that generate the CLI and configuration references.

Command handlers should send normal output through the `Printer` abstraction in
`crates/seal/src/printer.rs`. Use `tracing` for diagnostics controlled by `-v`; do not print
directly from handlers.

## Prerequisites

Install Rust with [rustup](https://rustup.rs/). Rustup will use the repository's pinned toolchain.

[nextest](https://nexte.st/) is recommended for the test suite:

```sh
cargo install cargo-nextest --locked
```

You can optionally install [prek](https://prek.j178.dev/) to run repository checks before each
commit:

```sh
uv tool install prek
prek install
```

Build the workspace from the repository root:

```sh
cargo build --workspace
```

## Development

Run the development CLI with Cargo:

```sh
cargo run -p seal -- --help
cargo run -p seal -- self version
```

Run the full test suite with nextest:

```sh
cargo nextest run --all-features
```

Use `cargo test` if nextest is unavailable. Pass arguments directly for focused iteration:

```sh
cargo nextest run -p seal
```

CLI integration tests live under
`crates/seal/tests/it/` and use the `seal_snapshot!` helper.

After updating snapshots, review the changes before accepting them. For the interactive review
workflow, install [`cargo-insta`](https://insta.rs/docs/cli/), run the relevant test, then run:

```sh
cargo insta review
```

Before opening a pull request, run the relevant tests and the full validation sweep:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
uvx prek run -a
```

## Documentation

Seal uses [Zensical](https://zensical.org/) for its documentation site. Prepare the generated home
page and build the site with:

```sh
uv run --script scripts/prepare_docs.py
uv run --isolated --with-requirements docs/requirements.txt zensical build
```

Use `zensical serve` instead of `zensical build` in the second command for a local development
server.

The CLI and configuration reference pages are generated. After changing CLI arguments,
configuration fields, defaults, or their source documentation, run:

```sh
cargo dev generate-all
```

Do not edit `docs/reference/cli.md` or `docs/reference/configuration.md` by hand.

## Release Process

Run the `Prepare release` workflow with the version bump to perform, such as `alpha` or an
explicit version. The workflow runs `seal bump <version>` and opens the release pull request.

To prepare a release locally, run:

```sh
cargo run -p seal -- bump <version>
```

Seal creates the release branch, commits and pushes the changes, and opens a pull request. Review
and merge it, then run the
[release workflow](https://github.com/MatthewMckee4/seal/actions/workflows/release.yml) with the
version tag without a leading `v`; the workflow builds artifacts, creates the GitHub release, and
publishes the documentation.

When changing GitHub Actions, run [`pinact`](https://github.com/suzuki-shunsuke/pinact) before
opening the pull request:

```sh
pinact run
```
