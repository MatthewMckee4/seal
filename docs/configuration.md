# Configuration

Seal uses a `seal.toml` file in your project root to configure release management behavior. This guide covers all available configuration options with examples.

## Quick Start

Create a minimal `seal.toml` to get started:

```toml
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
```

Run `seal validate` to check your configuration is valid.

## Configuration Reference

### `[release]` Section

All release configuration is defined under the `[release]` table.

#### `current-version` (required)

The current version of your project. Must follow semantic versioning (e.g., `1.2.3`).

```toml
[release]
current-version = "0.5.0"
```

Note: After bumping, you must manually update this value in `seal.toml` for the next release, or let your CI/CD do it.

#### `version-files` (optional, default: `[]`)

List of files where version strings should be updated. Can be simple paths or detailed configurations.

**Simple format** (auto-detects version patterns):

```toml
[release]
version-files = ["Cargo.toml", "package.json", "VERSION"]
```

**Detailed format** (custom search patterns and templates):

```toml
[[release.version-files]]
path = "version.sh"
search = "export VERSION=\"{version}\""

[[release.version-files]]
path = "include/version.h"
search = "#define VERSION \"{version}\""
version-template = "v{major}.{minor}.{patch}"
```

**Mixed format**:

```toml
[release]
version-files = [
    "Cargo.toml",
    { path = "docs/VERSION", search = "Version: {version}", version-template = "{major}.{minor}" }
]
```

See [Version File Customization](version-files.md) for detailed examples.

#### `commit-message` (optional, default: `"Release v{version}"`)

Template for the commit message when creating a release commit.

```toml
[release]
commit-message = "chore: release {version}"
```

The `{version}` placeholder is replaced with the new version number.

#### `branch-name` (optional, default: `"release/v{version}"`)

Template for the branch name created for the release.

```toml
[release]
branch-name = "releases/{version}"
```

#### `tag-format` (optional, default: `"v{version}"`)

Template for git tags created for releases.

```toml
[release]
tag-format = "{version}"
```

## Version File Configuration

### Auto-Detection

When using simple file paths, Seal automatically detects and updates these patterns:

- `version = "1.2.3"` (TOML files like Cargo.toml)
- `"version": "1.2.3"` (JSON files like package.json)
- `__version__ = "1.2.3"` (Python files)

### Custom Search Patterns

For non-standard formats, specify a search pattern with the `{version}` placeholder:

```toml
[[release.version-files]]
path = "Makefile"
search = "VERSION := {version}"
```

Seal will:
1. Replace `{version}` with the current version to find the line
2. Replace `{version}` with the new version to update it

**Important**: The search pattern must match exactly. If not found, the bump will fail.

### Version Templates

Customize how versions are formatted in specific files using these placeholders:

- `{major}` - Major version number (e.g., `1` in `1.2.3`)
- `{minor}` - Minor version number (e.g., `2` in `1.2.3`)
- `{patch}` - Patch version number (e.g., `3` in `1.2.3`)
- `{extra}` - Prerelease identifier (e.g., `-alpha.1` in `1.2.3-alpha.1`, empty for stable)

**Example: API version with only major.minor**:

```toml
[[release.version-files]]
path = "docs/api.txt"
search = "API Version: {version}"
version-template = "{major}.{minor}"
```

Bumping `1.2.3` → `1.3.0` updates the file to `API Version: 1.3`.

**Example: C header with v prefix**:

```toml
[[release.version-files]]
path = "include/version.h"
search = "#define VERSION \"{version}\""
version-template = "v{major}.{minor}.{patch}"
```

**Example: Prerelease support**:

```toml
[[release.version-files]]
path = "VERSION"
search = "version={version}"
version-template = "{major}.{minor}.{patch}{extra}"
```

When bumping `2.0.0-beta.1` → `2.0.0-beta.2`, this preserves the prerelease format.

## Complete Configuration Examples

### Minimal Configuration

```toml
[release]
current-version = "1.0.0"
version-files = ["Cargo.toml"]
```

### Standard Project

```toml
[release]
current-version = "2.1.5"
version-files = ["Cargo.toml", "package.json"]
commit-message = "Release v{version}"
branch-name = "release/v{version}"
tag-format = "v{version}"
```

### Multi-Language Project

```toml
[release]
current-version = "3.0.5"
commit-message = "chore: bump to {version}"
branch-name = "bump/{version}"
tag-format = "v{version}"

version-files = [
    "Cargo.toml",
    "package.json",
    { path = "version.py", search = "__version__ = '{version}'" },
    { path = "include/version.h", search = "#define VERSION \"{version}\"", version-template = "v{major}.{minor}.{patch}" }
]
```

### Documentation with Simplified Versions

```toml
[release]
current-version = "1.5.2"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

version-files = [
    "Cargo.toml",
    { path = "docs/VERSION", search = "Version: {version}", version-template = "{major}.{minor}" },
    { path = "README.md", search = "version `{version}`" }
]
```

## Validation

Validate your configuration before bumping:

```bash
seal validate
```

This checks:
- `seal.toml` exists and is valid TOML
- Required fields are present
- `{version}` placeholder exists in templates
- Version files are readable
- Current version is valid semver

## Best Practices

1. **Always validate first**: Run `seal validate` after editing `seal.toml`

2. **Keep it simple**: Use auto-detection for standard files when possible

3. **Be explicit with custom patterns**: Include enough context in search patterns to avoid false matches

4. **Version control your config**: Commit `seal.toml` to your repository

5. **Test in a branch**: Try `seal bump patch --no-push --no-pr` first to test locally

6. **Document custom templates**: Add comments in your TOML for complex configurations

```toml
[release]
current-version = "1.0.0"

# C header needs v prefix
[[release.version-files]]
path = "include/version.h"
search = "#define VERSION \"{version}\""
version-template = "v{major}.{minor}.{patch}"
```

## Troubleshooting

### "Failed to parse version bump argument"

The version bump argument (major/minor/patch/alpha/etc.) is invalid or the explicit version doesn't follow semver.

### "Search pattern not found in file"

The custom search pattern doesn't match any line in the file. Check:
- The pattern exactly matches what's in the file (quotes, spaces, etc.)
- The file contains the current version from `current-version`
- No typos in the search pattern

### "No version field found in file"

For files without a custom search pattern, none of the auto-detected patterns matched. Add a custom `search` pattern.

### "Version file not found"

The file path in `version-files` doesn't exist. Check:
- Path is relative to project root
- File exists
- No typos in the path

## Command Line Usage

### Bumping Versions

```bash
# Bump patch version (1.2.3 → 1.2.4)
seal bump patch

# Bump minor version (1.2.3 → 1.3.0)
seal bump minor

# Bump major version (1.2.3 → 2.0.0)
seal bump major

# Prerelease versions
seal bump major-alpha  # 1.2.3 → 2.0.0-alpha.1
seal bump minor-beta   # 1.2.3 → 1.3.0-beta.1
seal bump patch-rc     # 1.2.3 → 1.2.4-rc.1
seal bump alpha        # 1.2.3-alpha.1 → 1.2.3-alpha.2

# Explicit version
seal bump 2.0.0
```

### Flags

- `--no-push` - Don't push the branch to remote
- `--no-pr` - Don't create a pull request
- `-q, --quiet` - Suppress output
- `-v, --verbose` - Verbose output
- `--no-progress` - Disable progress indicators

## Migration from Other Tools

### From bump2version/bumpversion

**bump2version `.bumpversion.cfg`**:
```ini
[bumpversion]
current_version = 1.2.3
commit = True
tag = True

[bumpversion:file:setup.py]
search = version='{current_version}'
replace = version='{new_version}'
```

**Equivalent `seal.toml`**:
```toml
[release]
current-version = "1.2.3"

[[release.version-files]]
path = "setup.py"
search = "version='{version}'"
```

**Key differences**:
- Seal uses `{version}` instead of `{current_version}` and `{new_version}`
- No separate `replace` field needed - Seal uses the same search pattern
- Seal always commits and creates branches (use `--no-push` to work locally)

### From semantic-release

semantic-release is commit-based and determines versions automatically. Seal requires explicit version bumping:

```bash
# Instead of: npx semantic-release
seal bump minor  # or major/patch based on your changes
```

Configure version files similarly to how you'd configure semantic-release plugins:

```toml
[release]
current-version = "1.0.0"
version-files = ["package.json", "package-lock.json"]
```

## See Also

- [Version File Customization](version-files.md) - Detailed guide on version file options
- CLI Reference (coming soon) - Complete command reference