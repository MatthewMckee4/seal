# Seal - Foundational Issues

This document outlines the core issues needed to build seal's MVP.

---

## Issue #1: Configuration File Support (seal.toml)

**Priority**: Critical
**Status**: Not started

### Description

Implement configuration file support to allow users to customize seal's behavior per project.

### Behavior

**File Discovery**:
- Search for `seal.toml` from current directory up to git root
- Use defaults if not found
- Support `--config <path>` flag

**Configuration Schema**:
```toml
[release]
# Files containing version strings (auto-detected if omitted)
version-files = ["Cargo.toml"]

# Git commit message template
commit-message = "Release v{version}"

# Branch name template for release
branch-name = "release/v{version}"

# Git tag format
tag-format = "v{version}"
```

**Inferred from Git**:
- **Base branch**: Detected from `git symbolic-ref refs/remotes/origin/HEAD` (e.g., main, master)
- **Remote**: Uses "origin" or the remote tracked by current branch

**Validation**:
- Parse TOML with clear error messages
- Validate field types
- Warn on unknown fields

**Output**:
- Silent unless `--verbose` or error
- Verbose: `Loaded configuration from seal.toml`
- Error example: `Error in seal.toml:5 - 'version-files' must be an array`

### Acceptance Criteria

- [ ] Parses seal.toml from project root
- [ ] Uses sensible defaults when config missing
- [ ] Infers base branch from git remote HEAD
- [ ] Infers remote from git config or defaults to "origin"
- [ ] Validates configuration with helpful errors
- [ ] Warns on unknown fields
- [ ] Tests verify parsing and validation

---

## Issue #2: Version Management System

**Priority**: Critical
**Status**: Not started

### Description

Implement ecosystem-agnostic version detection, parsing, and bumping using semantic versioning.

### Behavior

**Version Detection**:
- Use `version-files` from config, or auto-detect:
  - `Cargo.toml`: `version = "X.Y.Z"`
  - `package.json`: `"version": "X.Y.Z"`
  - `pyproject.toml`: `version = "X.Y.Z"`
  - `VERSION` file: entire content
  - `*.csproj`: `<Version>X.Y.Z</Version>`
- Error if versions don't match across files

**Version Parsing**:
- Support semver: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]`
- Error on invalid format: `Invalid version '1.2' - must be MAJOR.MINOR.PATCH`

**Version Bumping**:
- `--bump major`: 1.2.3 → 2.0.0
- `--bump minor`: 1.2.3 → 1.3.0
- `--bump patch`: 1.2.3 → 1.2.4
- `--bump prerelease`: 1.2.3 → 1.2.4-rc.1 (or 1.2.4-rc.1 → 1.2.4-rc.2)
- `--version X.Y.Z`: Set explicit version
- Clears prerelease on major/minor/patch bumps
- Preserves build metadata

**File Updates**:
- Update all detected version files
- Preserve formatting (indentation, quotes)
- Only modify version string
- Create `.seal-backup` before writing

**Output**:
- `Current version: 1.2.3`
- `Bumping to 1.3.0`
- `Updated: Cargo.toml, package.json`
- Dry-run: `Would update version to 1.3.0`

### Acceptance Criteria

- [ ] Auto-detects versions in common file formats
- [ ] Errors on version mismatches
- [ ] Correctly bumps major, minor, patch, prerelease
- [ ] Preserves file formatting
- [ ] Creates backups before modifying
- [ ] Tests cover all bump types

---

## Issue #3: Release Command

**Priority**: Critical
**Status**: Not started

### Description

Implement `seal release` command that creates a release branch, bumps version, commits changes, and optionally creates a pull request.

### Behavior

**Command Syntax**:
```bash
seal release [OPTIONS]

Options:
  --bump <TYPE>        Version bump: major, minor, patch, prerelease
  --version <VERSION>  Set explicit version
  --message <MSG>      Custom commit message
  --pr                 Create pull request (requires gh CLI)
  --allow-dirty        Allow uncommitted changes
  --dry-run            Preview without making changes
```

**Workflow**:
1. Load seal.toml (or use defaults)
2. Infer base branch and remote from git
3. Verify git working directory is clean (unless `--allow-dirty`)
4. Detect current version
5. Calculate new version from `--bump` or `--version`
6. Create release branch: `release/v{version}`
7. Update version in all files
8. Commit changes: `Release v{version}`
9. Push branch to remote
10. Create PR if `--pr` flag set (using `gh pr create`)

**Interactive Mode**:
- If `--bump` not provided, prompt: `Version bump? [major/minor/patch/prerelease/custom]`
- Show preview: `1.2.3 → 1.3.0`
- Confirm: `Create release? [y/N]`

**Dry-Run Mode**:
- Print operations without executing:
  - `Would create branch: release/v1.3.0`
  - `Would update version: 1.2.3 → 1.3.0`
  - `Would commit: "Release v1.3.0"`
  - `Would push to origin`

**Output Example**:
```
Current version: 1.2.3
Creating release for v1.3.0

✓ Created branch: release/v1.3.0
✓ Updated version in: Cargo.toml, package.json
✓ Committed changes
✓ Pushed to origin

Next steps:
  • Review the changes
  • Create a pull request: gh pr create
  • After merge, run: seal tag v1.3.0
```

**Error Handling**:
- Check git is installed and repo exists
- Error if working directory is dirty (unless `--allow-dirty`)
- Error if branch already exists
- Clear error messages for git failures

### Acceptance Criteria

- [ ] Creates release branch with version bump
- [ ] Updates version files and commits
- [ ] Pushes branch to remote
- [ ] Creates PR when `--pr` flag provided
- [ ] Interactive mode prompts for bump type
- [ ] Dry-run shows operations without executing
- [ ] Respects seal.toml configuration
- [ ] Uses Printer abstraction for output
- [ ] Tests cover interactive, declarative, dry-run modes

---

## Issue #4: Tag Command

**Priority**: Critical
**Status**: Not started

### Description

Implement `seal tag` command for tagging and pushing to main after a release PR is merged.

### Behavior

**Command Syntax**:
```bash
seal tag [VERSION] [OPTIONS]

Arguments:
  [VERSION]  Version to tag (auto-detected if omitted)

Options:
  --format <FORMAT>  Tag format (default: from seal.toml or "v{version}")
  --message <MSG>    Tag message (default: "Release {version}")
  --push             Push tag to remote
  --dry-run          Preview without making changes
```

**Workflow**:
1. Load seal.toml
2. Infer base branch from git
3. Detect current version from files (or use provided VERSION)
4. Verify on base branch (e.g., main)
5. Verify working directory is clean
6. Create annotated git tag: `v{version}`
7. Push tag if `--push` flag set or prompt user

**Auto-detection**:
- If VERSION not provided, detect from version files
- Use `tag-format` from seal.toml or default `v{version}`

**Output**:
```
Tagging version 1.3.0

✓ Created tag: v1.3.0
Push tag to origin? [y/N]: y
✓ Pushed tag to origin

Release v1.3.0 published!
```

**Dry-Run Mode**:
```
Would create tag: v1.3.0
Would push tag to origin
```

**Error Handling**:
- Error if not on base branch (detected from git)
- Error if tag already exists
- Error if working directory is dirty
- Error if version not detected (and not provided)
- Error if cannot detect base branch

### Acceptance Criteria

- [ ] Creates annotated git tag
- [ ] Auto-detects version from files
- [ ] Accepts explicit version argument
- [ ] Prompts before pushing (unless `--push`)
- [ ] Validates on correct branch
- [ ] Dry-run shows operations
- [ ] Uses configured tag format
- [ ] Tests verify tag creation and push

---

## Implementation Order

1. **Issue #1: Configuration File Support** - Foundation
2. **Issue #2: Version Management System** - Core functionality
3. **Issue #3: Release Command** - Creates release branch and PR
4. **Issue #4: Tag Command** - Tags after merge

## MVP Definition

The MVP consists of all 4 issues, providing:
- Customizable configuration (seal.toml)
- Auto-inference of base branch and remote from git
- Version detection and bumping (ecosystem-agnostic)
- PR-based release workflow
- Git tagging after merge
- Interactive and declarative modes
- Dry-run support
