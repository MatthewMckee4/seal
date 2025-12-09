# Seal

An extremely fast release management tool, written in Rust.

> [!WARNING]
>
> Seal is not yet ready for production use.
>
> You may run into bugs, missing features, and other issues.

## What is Seal?

Seal is a release management tool that can be used to automate version updating.

There are other tools out there for version bumping and changelog generation and management,
but often tools only support one of these.

Seal aims to provide a unified solution for both version bumping and changelog management.

## Installation

Install uv with our standalone installers:

```shell
# On macOS and Linux.
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MatthewMckee4/seal/releases/download/0.0.1-alpha.3/seal-installer.sh | sh
```

```shell
# On Windows.
powershell -ExecutionPolicy Bypass -c "irm https://github.com/MatthewMckee4/seal/releases/download/0.0.1-alpha.3/seal-installer.ps1 | iex"
```

We do not (yet) have support for installation from other sources, like PyPI or cargo.

## Documentation

seal's documentation is available at [matthewmckee4.github.io/seal](https://matthewmckee4.github.io/seal/)

## Acknowledgements

I'd like to thank the [Astral team](https://github.com/astral-sh) for all of their contributions to the Rust ecosystem.

Particularly, the projects [uv](https://github.com/astral-sh/uv) and [ruff](https://github.com/astral-sh/ruff).

## License

Seal is licensed under the MIT License.

We also include the [uv MIT license](https://github.com/MatthewMckee4/seal/blob/main/licenses/astral.LICENSE-MIT), as we often
take inspiration or code snippets from the [uv](https://github.com/astral-sh/uv) repository.
