# Seal

[![codecov](https://codecov.io/gh/MatthewMckee4/seal/graph/badge.svg?token=URQ3YZHYDK)](https://codecov.io/gh/MatthewMckee4/seal)

Seal is a release-management tool, written in Rust. It keeps version updates, changelog generation,
release branches, commits, and release notes in one CLI.

> [!WARNING]
>
> Seal is in alpha and is not yet ready for production use. Expect missing features and breaking
> changes.

## Features

- Update versions in plain-text files, TOML fields, and files selected by glob patterns.
- Preview every file change before applying it, or use `--dry-run` in automation.
- Create release branches and commits, run pre-commit commands, and push the result.
- Build changelogs from merged GitHub pull requests, grouped by labels.
- Produce JSON release metadata from the latest changelog section.

## Installation

Install Seal with the standalone installer for your platform:

```sh
# macOS and Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MatthewMckee4/seal/releases/download/0.0.1-alpha.7/seal-installer.sh | sh
```

```powershell
# Windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/MatthewMckee4/seal/releases/download/0.0.1-alpha.7/seal-installer.ps1 | iex"
```

Prebuilt archives and checksums are available from the
[GitHub Releases](https://github.com/MatthewMckee4/seal/releases). Seal is not currently
published to crates.io or other package registries.

## Getting Started

Create `seal.toml` in your project root:

```toml
[release]
current-version = "0.1.0"
version-files = [
    { path = "Cargo.toml", format = "toml", field = "package.version" },
]
```

Validate the configuration and preview a patch release:

```console
seal validate config
seal bump patch --dry-run
```

Run `seal bump patch` to review and apply the changes. See the
[getting-started guide](https://matthewmckee4.github.io/seal/get-started/quick-start/) for branch,
commit, and push configuration.

## Documentation

The full documentation is available at
[matthewmckee4.github.io/seal](https://matthewmckee4.github.io/seal/).

## Contributing

Contributions are welcome. See
[CONTRIBUTING.md](https://github.com/MatthewMckee4/seal/blob/main/CONTRIBUTING.md) for setup and
development instructions.

## Support

Use [GitHub issues](https://github.com/MatthewMckee4/seal/issues) for bug reports and feature
requests.

Report security issues privately; see
[SECURITY.md](https://github.com/MatthewMckee4/seal/blob/main/SECURITY.md).

## Acknowledgements

Seal takes inspiration from the Rust tooling built by the
[Astral team](https://github.com/astral-sh), particularly
[uv](https://github.com/astral-sh/uv) and [Ruff](https://github.com/astral-sh/ruff).

## License

Seal is licensed under the [MIT License](https://github.com/MatthewMckee4/seal/blob/main/LICENSE).
The repository also includes Astral's
[MIT license](https://github.com/MatthewMckee4/seal/blob/main/licenses/astral.LICENSE-MIT) because
some implementation ideas and snippets are derived from `uv`.
