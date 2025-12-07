# Version File Customization

Seal provides customization options for managing version strings across different files in your project.

## Basic Configuration

The simplest way to specify version files is using an array of file paths:

```toml
[release]
current-version = "1.2.3"
version-files = ["Cargo.toml", "package.json", "VERSION"]
```

Seal will automatically detect and update common version patterns in these files:
- `version = "1.2.3"` (TOML, like Cargo.toml)
- `"version": "1.2.3"` (JSON, like package.json)
- `__version__ = "1.2.3"` (Python)

## Custom Search Patterns

For files with non-standard version formats, you can specify custom search patterns:

```toml
[release]
current-version = "2.5.0"

[[release.version-files]]
path = "version.sh"
search = "export VERSION=\"{version}\""

[[release.version-files]]
path = "config.py"
search = "APP_VERSION = '{version}'"
```

The `{version}` placeholder in the search pattern will be replaced with the current version when searching, and with the new version when updating.

## Version Templates

Version templates allow you to customize how the version string is formatted in specific files. This is useful when different files need different version formats:

```toml
[release]
current-version = "1.2.3"

[[release.version-files]]
path = "VERSION.txt"
search = "Version: {version}"
version-template = "{major}.{minor}"

[[release.version-files]]
path = "full_version.txt"
search = "Full: {version}"
# This file will use the complete version (1.2.3)
```

When you bump from `1.2.3` to `1.3.0`:
- `VERSION.txt` will contain `Version: 1.3` (only major.minor)
- `full_version.txt` will contain `Full: 1.3.0` (complete version)

### Available Template Placeholders

- `{major}` - The major version number (e.g., `1` in `1.2.3`)
- `{minor}` - The minor version number (e.g., `2` in `1.2.3`)
- `{patch}` - The patch version number (e.g., `3` in `1.2.3`)
- `{extra}` - The prerelease identifier (e.g., `-alpha.1` in `1.2.3-alpha.1`, empty for stable versions)

### Template Examples

**Major.Minor format:**
```toml
version-template = "{major}.{minor}"
# 1.2.3 → 1.2
```

**With prefix:**
```toml
version-template = "v{major}.{minor}.{patch}"
# 1.2.3 → v1.2.3
```

**With prerelease support:**
```toml
version-template = "{major}.{minor}.{patch}{extra}"
# 1.2.3 → 1.2.3
# 2.0.0-alpha.1 → 2.0.0-alpha.1
```

**Custom prerelease format:**
```toml
version-template = "{major}.{minor}.{patch}-{extra}"
# 2.0.0-alpha.1 → 2.0.0--alpha.1 (note: adds hyphen even when extra is empty)
```

## Mixed Configuration

You can mix simple file paths with detailed configurations:

```toml
[release]
current-version = "1.5.2"

version-files = [
    "Cargo.toml",  # Uses auto-detection
    { path = "version.txt", search = "Version: {version}", version-template = "{major}.{minor}" },
    { path = "full_version.txt", search = "Full: {version}" }
]
```

## Complete Example

Here's a comprehensive example showing different customization options:

```toml
[release]
current-version = "3.0.5"
commit-message = "Release {version}"
branch-name = "release/{version}"
tag-format = "v{version}"

version-files = [
    # Standard files with auto-detection
    "Cargo.toml",
    "package.json",
    
    # C header with custom pattern and prefix
    { 
        path = "include/version.h",
        search = "#define VERSION \"{version}\"",
        version-template = "v{major}.{minor}.{patch}"
    },
    
    # Shell script with custom pattern
    {
        path = "scripts/version.sh",
        search = "export APP_VERSION=\"{version}\""
    },
    
    # Documentation with major.minor only
    {
        path = "docs/VERSION",
        search = "Version: {version}",
        version-template = "{major}.{minor}"
    },
    
    # README with custom pattern (multiple occurrences)
    {
        path = "README.md",
        search = "version `{version}`"
    }
]
```

## Pattern Matching Behavior

### Multiple Occurrences

If your search pattern appears multiple times in a file, **all occurrences will be replaced**:

```markdown
# My Project

Current version `1.0.0` is stable.

Install version `1.0.0` with npm.
```

With `search = "version `{version}`"`, both occurrences will be updated when bumping.

### Pattern Not Found

If a search pattern is not found in a file, the bump command will fail with an error:

```
error: Search pattern not found in file. Expected: VERSION=1.0.0
```

This ensures you don't accidentally skip files due to typos or file changes.

### Auto-detection Fallback

For simple file paths without a custom search pattern, Seal tries common patterns in order:
1. `version = "..."`
2. `"version": "..."`
3. `__version__ = "..."`

If none match, the bump will fail.

## Best Practices

1. Use auto-detection when possible - it's simpler and covers common cases
2. Be specific with search patterns - include enough context to avoid false matches
3. Test custom patterns - run `seal validate` to ensure your configuration works
4. Document custom templates - add comments in your seal.toml for complex configurations
5. Keep templates consistent - use the same template format across similar files

## Troubleshooting

### Pattern not found error
- Verify the pattern exactly matches what's in the file (check quotes, spaces, etc.)
- Ensure the file contains the current version from `current-version`
- Check for hidden characters or encoding issues

### Wrong replacements
- Make your search pattern more specific by including more context
- Use version templates if the version format differs from the canonical version

### Prerelease versions
- When using prereleases (e.g., `1.0.0-alpha.1`), include the `{extra}` placeholder in templates if needed
- The `{extra}` placeholder includes the leading hyphen (e.g., `-alpha.1`)