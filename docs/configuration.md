# Configuration

Seal uses a `seal.toml` file in your project root to configure release management behavior.

## Quick Start

Minimal configuration:

```toml
[release]
current-version = "1.0.0"
```

This will only update `seal.toml` when bumping. Add more options as needed.

## Configuration Reference

### `current-version` (required)

The current version of your project. Must follow semantic versioning.

```toml
[release]
current-version = "1.2.3"
```

### `version-files` (optional)

List of files where version strings should be updated. If not specified, only `seal.toml` is updated.

**Simple paths** (auto-detects common patterns):

```toml
[release]
version-files = ["Cargo.toml", "package.json"]
```

Auto-detected patterns:
- `version = "1.2.3"` (TOML files)
- `"version": "1.2.3"` (JSON files)
- `__version__ = "1.2.3"` (Python files)

**Custom search patterns:**

```toml
[[release.version-files]]
path = "version.sh"
search = "export VERSION=\"{version}\""

[[release.version-files]]
path = "Makefile"
search = "VERSION := {version}"
```

**Version templates** (customize format per file):

```toml
[[release.version-files]]
path = "docs/VERSION"
search = "Version: {version}"
version-template = "{major}.{minor}"  # Only major.minor

[[release.version-files]]
path = "version.h"
search = "#define VERSION \"{version}\""
version-template = "v{major}.{minor}.{patch}"  # With prefix
```

Template placeholders:
- `{major}` - Major version (1 in 1.2.3)
- `{minor}` - Minor version (2 in 1.2.3)
- `{patch}` - Patch version (3 in 1.2.3)
- `{extra}` - Prerelease (alpha.1 in 1.2.3-alpha.1, empty for stable)

### `commit-message` (optional)

Template for the commit message. If not set, no commit is created.

```toml
[release]
commit-message = "Release {version}"
```

The `{version}` placeholder is replaced with the new version.

### `branch-name` (optional)

Template for the release branch. If not set, no branch is created.

```toml
[release]
branch-name = "release/{version}"
```

### `tag-format` (optional)

Template for git tags.

```toml
[release]
tag-format = "v{version}"
```

### `push` (optional, default: `false`)

Whether to push the branch to remote. Requires `branch-name` to be set.

```toml
[release]
push = true
```

### `create-pr` (optional, default: `false`)

Whether to create a pull request. Requires both `branch-name` and `push = true`. Also requires `gh` CLI.

```toml
[release]
create-pr = true
```

### `confirm` (optional, default: `true`)

Whether to prompt for confirmation before making changes.

```toml
[release]
confirm = false  # Skip confirmation, useful for CI/CD
```

## Complete Examples

### Minimal (files only, no git)

```toml
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml", "package.json"]
```

### Local commits only

```toml
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
commit-message = "Bump to {version}"
```

### Local branch only (no commit)

```toml
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
branch-name = "release/{version}"
```

### Full automated workflow

```toml
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml", "package.json"]
branch-name = "release/{version}"
commit-message = "Release {version}"
tag-format = "v{version}"
push = true
create-pr = true
confirm = false
```

### Multi-language project

```toml
[release]
current-version = "2.0.0"
branch-name = "release/{version}"
commit-message = "chore: bump to {version}"
push = true

version-files = [
    "Cargo.toml",
    "package.json",
    { path = "version.py", search = "__version__ = '{version}'" },
    { path = "include/version.h", search = "#define VERSION \"{version}\"", version-template = "v{major}.{minor}.{patch}" }
]
```

## Usage

### Bump versions

```bash
# Semantic version bumps
seal bump patch   # 1.2.3 → 1.2.4
seal bump minor   # 1.2.3 → 1.3.0
seal bump major   # 1.2.3 → 2.0.0

# Prerelease versions
seal bump major-alpha  # 1.2.3 → 2.0.0-alpha.1
seal bump minor-beta   # 1.2.3 → 1.3.0-beta.1
seal bump patch-rc     # 1.2.3 → 1.2.4-rc.1
seal bump alpha        # 1.2.3-alpha.1 → 1.2.3-alpha.2

# Explicit version
seal bump 2.0.0
seal bump 2.0.0-beta.1

# Dry run (preview changes)
seal bump minor --dry-run
```

### Validation

```bash
seal validate config          # Validate seal.toml
seal validate project         # Validate full project
```

## Validation Rules

Configuration validation ensures:
- `push = true` requires `branch-name` to be set
- `create-pr = true` requires both `branch-name` and `push = true`
- All templates contain `{version}` placeholder
- Version is valid semver

## Troubleshooting

### "Search pattern not found in file"

The custom search pattern doesn't match the file content. Check:
- Pattern exactly matches what's in the file
- File contains the current version from `current-version`
- No typos in the search pattern

### "No version field found in file"

For files without custom search patterns, none of the auto-detected patterns matched. Add a custom `search` pattern.

### "Version file not found"

The file path in `version-files` doesn't exist. Paths are relative to project root.

### "release.push = true requires branch-name to be set"

You cannot push without creating a branch. Either:
- Set `branch-name = "release/{version}"`
- Set `push = false`

### "release.create-pr = true requires both branch-name and push = true"

Creating PRs requires pushing a branch. Set both:
- `branch-name = "release/{version}"`
- `push = true`
