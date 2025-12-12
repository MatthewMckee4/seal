# Migrating from Rooster to Seal

Migrate your rooster configuration to seal:

```bash
seal migrate rooster -i pyproject.toml -o seal.toml
```

## Supported Features

**Changelog**: `ignore_labels`, `changelog_ignore_authors`, `changelog_contributors`, `section_labels` are automatically migrated.

**Version files**: Automatically migrated with a placeholder `current-version = "0.0.0"` - update this to your actual version.

## Unsupported Features

- `major_labels`, `minor_labels`, `default_bump_type` - Seal requires explicit version specification (`seal bump 1.2.3`)
- `submodules` - Configure members separately in `[members]` section
- `require_labels`, `trim_title_prefixes` - No equivalent
- `version_tag_prefix` - Seal always uses `v` prefix

## Migration Steps

1. **Run migration**:

   ```bash
   seal migrate rooster -i pyproject.toml -o seal.toml
   ```

1. **Update current version** (if version files were migrated):

   ```toml
   [release]
   current-version = "0.5.0"  # Change from placeholder "0.0.0"
   ```

1. **Validate**: `seal validate config`

## Example

**Before** (pyproject.toml):

```toml
[tool.rooster]
changelog_ignore_labels = ["internal", "ci"]
version_files = ["Cargo.toml", "README.md"]

[tool.rooster.section-labels]
"Breaking changes" = ["breaking"]
```

**After** (seal.toml):

```toml
[release]
current-version = "0.0.0"  # Update to your actual version
version-files = [
    "Cargo.toml",
    "README.md",
]
push = false
create-pr = false
confirm = true

[changelog]
ignore-labels = [
    "internal",
    "ci",
]

[changelog.section-labels]
"Breaking changes" = ["breaking"]
```

## Key Differences

- **Rooster**: Auto-determines version from labels
- **Seal**: Explicit version specification (`seal bump minor`)

See `seal help` for more information.
