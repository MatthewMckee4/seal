# Bumping Versions

Seal updates `release.current-version` and every configured version file in one operation. It shows
the complete diff before writing anything.

## Basic Version File

Given this configuration:

```toml title="seal.toml"
[release]
current-version = "0.0.1"
version-files = ["README.md"]
```

And a `README.md` containing:

```markdown
# My Project (0.0.1)
```

Preview and apply a patch release with:

```console
seal bump patch --dry-run
seal bump patch
```

Seal replaces `0.0.1` with `0.0.2` in `README.md` and updates `current-version` in `seal.toml`.

## Structured and Targeted Replacements

Use a TOML field when only one value should change, and a search template for a precise text
replacement:

```toml title="Cargo.toml"
[package]
name = "my-app"
version = "0.0.1"
```

```rust title="src/version.rs"
pub const VERSION: &str = "0.0.1";
```

```toml title="seal.toml"
[release]
current-version = "0.0.1"
version-files = [
    { path = "Cargo.toml", field = "package.version", format = "toml" },
    { path = "src/version.rs", search = "pub const VERSION: &str = \"{version}\";" },
]
```

Paths may be glob patterns. The
[configuration reference](../reference/configuration.md#release_version-files) documents every
supported version-file form.

## Version Arguments

Seal accepts stable increments, pre-release increments, and explicit semantic versions:

```console
seal bump major
seal bump minor
seal bump patch
seal bump alpha
seal bump minor-beta
seal bump 2.0.0-rc.1
```

An explicit version must be newer than `current-version`.

## Release Branches and Commits

Configure branch and commit templates to run the Git workflow after files are updated:

```toml title="seal.toml"
[release]
current-version = "0.0.1"
version-files = ["README.md"]
branch-name = "release/v{version}"
commit-message = "Release v{version}"
push = true
```

Both templates must contain `{version}`. `push = true` requires `branch-name`.

## Release Pull Requests

Add a `[release.pull-request]` table to open a pull request after Seal pushes the release branch:

```toml title="seal.toml"
[release]
current-version = "1.2.3"
commit-message = "Release v{version}"
branch-name = "release/v{version}"
push = true

[release.pull-request]
title = "Release v{version}"
body = "Prepare release v{version}."
base = "main"
draft = true
```

The table itself enables pull-request creation and requires `commit-message`, `branch-name`, and
`push = true`. `title` defaults to the resolved commit message, `body` defaults to the generated
changelog section body or an empty string, `base` defaults to the branch from which the release
branch was created, and `draft` defaults to `false`. The title and body support `{version}`.

Seal updates an existing open pull request with the same head and base branches instead of creating
a duplicate. Creating or updating a pull request requires `GITHUB_TOKEN` or `GH_TOKEN` with access
to the repository.

## Pre-Commit Commands

Use `pre-commit-commands` to run formatters or validation after Seal stages version changes and
before it creates the release commit:

```toml title="seal.toml"
[release]
current-version = "0.0.1"
commit-message = "Release {version}"
pre-commit-commands = [
    "cargo fmt --check",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
]
```

Seal stages changes again after the commands, so files updated by a formatter are included in the
release commit.

By default, a failing command aborts the release. To log the failure and continue instead:

```toml title="seal.toml"
[release]
current-version = "0.0.1"
commit-message = "Release {version}"
pre-commit-commands = ["cargo fmt --check"]
on-pre-commit-failure = "continue"
```
