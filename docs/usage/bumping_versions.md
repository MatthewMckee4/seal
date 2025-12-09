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

Preview of changes:
-------------------

diff --git a/my_app/__init__.py b/my_app/__init__.py
--- a/my_app/__init__.py
+++ b/my_app/__init__.py
@@ -1 +1 @@
-__version__ = "0.0.1"
+__version__ = "0.0.2"

diff --git a/pyproject.toml b/pyproject.toml
--- a/pyproject.toml
+++ b/pyproject.toml
@@ -1,5 +1,5 @@
 [project]
 name = "my_app"
-version = "0.0.1"
+version = "0.0.2"
 description = "My App"
 requires-python = ">=3.13"

diff --git a/seal.toml b/seal.toml
--- a/seal.toml
+++ b/seal.toml
@@ -1,5 +1,5 @@
 [release]
-current-version = "0.0.1"
+current-version = "0.0.2"

 version-files = [
     { path = "my_app/__init__.py", search = "__version__ = \"{version}\"" },
Skipping changelog update because no `[changelog]` section was found in the configuration.

Changes to be made:
  - Update `/my_app/__init__.py`
  - Update `/pyproject.toml`
  - Update `/seal.toml`

Note: No branch or commit will be created (branch-name and commit-message not configured)

Proceed with these changes? (y/n):
```

Once you proceed with `y`, the changes will be applied.
