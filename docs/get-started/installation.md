# Installation

Seal is currently distributed through GitHub Releases.

## Standalone Installer

On macOS and Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MatthewMckee4/seal/releases/download/0.0.1-alpha.8/seal-installer.sh | sh
```

On Windows:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/MatthewMckee4/seal/releases/download/0.0.1-alpha.8/seal-installer.ps1 | iex"
```

The installer selects the correct prebuilt archive and places `seal` on your path. Confirm the
installation with:

```console
seal self version
```

Prebuilt archives and SHA-256 checksums are available on the
[GitHub Releases page](https://github.com/MatthewMckee4/seal/releases).

> **Warning:** Seal is in alpha. Releases may contain breaking changes, and Seal is not yet
> published to crates.io or other package registries.

Continue with the [quick start](quick-start.md) to configure a project.

## GitHub Actions

Use the composite setup action with an explicit release version. Pin the action reference to a
reviewed commit SHA, and pin the installed Seal version separately:

```yaml
permissions: {}

steps:
  - uses: MatthewMckee4/seal/.github/actions/setup-seal@<full-commit-sha>
    with:
      version: 0.0.1-alpha.8
```

The setup action only installs Seal and requires no token or write permission. For a read-only
release preview, run `seal bump patch --dry-run`. Keep any permissions for release branches,
commits, pushes, or pull requests in the consuming workflow.
