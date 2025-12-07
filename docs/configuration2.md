<!-- WARNING: This file is auto-generated (cargo run -p seal_dev generate-all). Update the doc comments in 'crates/seal_project/src/config.rs' if you want to change anything here. -->

# Configuration

Seal configuration is defined in a `seal.toml` file in the root of your project.

## `[release]`

Release management configuration.

### `current-version`

**Required**

**Type**: `string`

The current version of the project.

**Example**:

```toml
[release]
current-version = "1.2.3"
```

---

### `version-files`

**Optional**

**Type**: `array` of strings or objects

List of files to update with the new version. Can be simple file paths or detailed configurations with search patterns.

**Example (simple)**:

```toml
[release]
version-files = ["Cargo.toml", "package.json"]
```

**Example (detailed)**:

```toml
[[release.version-files]]
path = "version.sh"
search = 'export VERSION="{version}"'

[[release.version-files]]
path = "Cargo.toml"
search = 'version = "{version}"'
version-template = "{version}"
```

---

### `commit-message`

**Optional**

**Type**: `string`

Commit message template for version bump commits. Must contain `{version}` placeholder.

**Example**:

```toml
[release]
commit-message = "chore: release v{version}"
```

---

### `branch-name`

**Optional**

**Type**: `string`

Branch name template for release branches. Must contain `{version}` placeholder. Required if `push` or `create-pr` is true.

**Example**:

```toml
[release]
branch-name = "release-{version}"
```

---

### `tag-format`

**Optional**

**Type**: `string`

Git tag format template. Must contain `{version}` placeholder.

**Example**:

```toml
[release]
tag-format = "v{version}"
```

---

### `push`

**Optional**

**Type**: `boolean`

**Default**: `false`

Whether to push the release branch to the remote repository. Requires `branch-name` to be set.

**Example**:

```toml
[release]
push = true
```

---

### `create-pr`

**Optional**

**Type**: `boolean`

**Default**: `false`

Whether to create a pull request for the release. Requires both `branch-name` and `push` to be set.

**Example**:

```toml
[release]
create-pr = true
```

---

### `confirm`

**Optional**

**Type**: `boolean`

**Default**: `true`

Whether to prompt for confirmation before making changes.

**Example**:

```toml
[release]
confirm = false
```

