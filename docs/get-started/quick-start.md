# Quick Start

Seal reads `seal.toml` from the current project directory. A minimal configuration tracks the
current version and the files containing it.

## Configure a Project

Create `seal.toml` in the project root:

```toml
[release]
current-version = "0.1.0"
version-files = [
    { path = "Cargo.toml", format = "toml", field = "package.version" },
]
```

This updates `package.version` without replacing unrelated occurrences of `0.1.0` in
`Cargo.toml`. Validate the configuration before the first release:

```console
seal validate config
```

Commit `seal.toml` and any version-file changes before previewing; version bumps require a clean
working tree and index by default.

## Preview a Version Bump

Preview a patch release without modifying files:

```console
seal bump patch --dry-run
```

Seal accepts `major`, `minor`, `patch`, pre-release bumps such as `alpha` or `minor-beta`, and
explicit semantic versions such as `1.0.0-rc.1`.

## Apply the Release

Run the command without `--dry-run`:

```console
seal bump patch
```

Seal shows the proposed diff and asks for confirmation before writing files. To prepare a release
branch and commit in the same operation, add templates containing `{version}`:

```toml
[release]
current-version = "0.1.0"
version-files = [
    { path = "Cargo.toml", format = "toml", field = "package.version" },
]
branch-name = "release/v{version}"
commit-message = "Release v{version}"
```

Set `push = true` to push the configured release branch. Seal rejects `push = true` unless a branch
name is configured.

See [Bumping Versions](../usage/bumping_versions.md) for other file formats and pre-commit commands,
or the [configuration reference](../reference/configuration.md) for every option.
