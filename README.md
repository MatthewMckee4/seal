# Seal

An extremely fast release management tool, written in Rust.

## What is Seal?

Seal unifies the release cycle into a single command.

## The Problem

Often release management is fragmented.
Seal aims to solve this by providing one tool that handles the entire release cycle.

## Status

Early development - not yet functional

## Vision

Seal aims to make releases as simple as running a single command,
with sensible defaults that work for most projects and clear configuration for those that need it.

## Features

### Version File Customization

Seal provides powerful options for managing version strings across different files:

- **Auto-detection** - Automatically finds and updates common version patterns (Cargo.toml, package.json, Python files)
- **Custom search patterns** - Define exact patterns to match in your files
- **Version templates** - Customize how versions are formatted per file (e.g., `v1.2.3`, `1.2`, `{major}.{minor}.{patch}`)
- **Mixed configurations** - Combine simple paths with detailed configurations

Example configuration:

```toml
[release]
current-version = "1.2.3"

version-files = [
    "Cargo.toml",  # Auto-detected
    { path = "version.h", search = "#define VERSION \"{version}\"", version-template = "v{major}.{minor}.{patch}" },
    { path = "docs/VERSION", search = "Version: {version}", version-template = "{major}.{minor}" }
]
```

For detailed documentation, see [docs/version-files.md](docs/version-files.md).

## Configuration

Create a `seal.toml` in your project root:

```toml
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml", "package.json"]
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"
```
