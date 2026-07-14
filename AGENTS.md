# Seal Repository

Seal is a Rust workspace for a release-management CLI. Read `CONTRIBUTING.md` before changing
files; it contains the current crate map, setup commands, documentation workflow, and release
process.

## Running Tests

Run the test suite with nextest:

```sh
cargo nextest run --all-features
```

Use `cargo test` if nextest is unavailable. Prefer a package or test filter while iterating:

```sh
cargo nextest run -p seal
```

Run the CLI from the workspace root:

```sh
cargo run -p seal -- --help
```

Run Clippy with the same strictness as CI:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Run `uvx prek run -a` at the end of every task. During iteration, use
`uvx prek run --files <path1> <path2>` with every changed file so hook runs do not depend on staged
state.

## Snapshots and Generated Files

Prefer integration tests under `crates/seal/tests/it/` when behavior crosses crate or CLI
boundaries. Command integration tests should use the existing `seal_snapshot!` helper.

Review snapshot changes before accepting them and check for pending `.snap.new` files before
finishing.

Run the following command after changing configuration options, CLI arguments, or anything else
that feeds the generated reference pages under `docs/reference/`:

```sh
cargo dev generate-all
```

Do not hand-edit generated reference pages.

## Development Guidelines

- Test every behavior change. If the relevant tests were not run, the change is not done.
- Look for existing utilities and neighboring patterns before writing new code.
- Keep visibility narrow unless another workspace crate genuinely needs the item.
- Keep Rust imports at the top of the file rather than inside functions.
- Avoid `panic!`, `unreachable!`, `.unwrap()`, unsafe code, and Clippy ignores. Encode constraints in
  the type system instead.
- Prefer `if let` for fallibility and let chains over nested `if let` statements when clearer.
- Prefer `#[expect(...)]` over `#[allow(...)]` when a lint must be suppressed.
- Use comments for invariants and unusual decisions, not to narrate code.
- Route command output through `Printer` and diagnostics through `tracing`; do not add direct print
  calls to command handlers.
- Consider whether a change needs a guide or regenerated reference page. New flags, changed
  defaults, configuration changes, and user-visible workflows usually do.

## Pull Requests

Use the pull request template and add relevant labels. Write the description in concise prose
paragraphs, with code examples only when they help the reviewer.
