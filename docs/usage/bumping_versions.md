# Bumping Versions

## Example

Here, we see a very small example of how to bump versions using seal.

Below is a very basic configuration file for bumping versions.

```text title="seal.toml"
[release]
current-version = "0.0.1"

version-files = [
    "README.md",
]
```

If you had a `README.md` file like this:

```markdown
# My Project (0.0.1)
```

With this setup, you can bump the version by running:

```shell
seal bump patch
```

Which will update your `README.md` file to:

```markdown
# My Project (0.0.2)
```

And update your `seal.toml` file to:

```toml
[release]
current-version = "0.0.2"

version-files = [
    "README.md",
]
```

## Other Version Files

If we were working in a Python project like this:

```python title="my_app/__init__.py"
__version__ = "0.0.1"
```

```toml title="pyproject.toml"
[project]
name = "my_app"
version = "0.0.1"
description = "My App"
requires-python = ">=3.13"
dependencies = []
```

```toml title="seal.toml"
[release]
current-version = "0.0.1"

version-files = [
    { path = "my_app/__init__.py", search = "__version__ = \"{version}\"" },
    { path = "pyproject.toml", field = "project.version", format = "toml" },
]
```

When you run `seal bump patch`, you will see the following output.

```text
Bumping version from 0.0.1 to 0.0.2

...

Proceed with these changes? (y/n):
```

Once you proceed with `y`, the changes will be applied.

## Pre-Commit Commands

You can configure commands to run before committing using the `pre-commit-commands` option.
These commands run after `git add -A` stages your version changes, allowing you to run
formatters, linters, or other tools. A second `git add -A` runs after the pre-commit
commands to stage any changes they make.

```toml title="seal.toml"
[release]
current-version = "0.0.1"
commit-message = "Release {version}"
pre-commit-commands = ["cargo fmt", "npm run lint:fix"]
```

When you run `seal bump patch`, the command sequence will be:

1. `git add -A` (stage version file changes)
1. `cargo fmt` (format code)
1. `npm run lint:fix` (fix lint issues)
1. `git add -A` (stage any changes from pre-commit commands)
1. `git commit -m "Release 0.0.2"`

### Failure Handling

By default, if any pre-commit command fails (exits with non-zero status), the release
process will abort. You can change this behavior with `on-pre-commit-failure`:

```toml title="seal.toml"
[release]
current-version = "0.0.1"
commit-message = "Release {version}"
pre-commit-commands = ["cargo fmt --check", "cargo clippy"]
on-pre-commit-failure = "continue"  # or "abort" (default)
```

With `continue`, failing commands will log a warning but the release will proceed.
